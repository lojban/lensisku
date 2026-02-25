import { jwtDecode } from "jwt-decode";
import { reactive, provide, inject } from "vue";
import { useRouter } from "vue-router";
import { setAuthInstance, api, performBackendLogout, mergeProgress } from "@/api";
import { getAllProgressForMerge, clearAfterMerge } from "@/composables/useAnonymousProgress";

const authKey = Symbol();

const REFRESH_MARGIN = 5 * 60; // 5 minutes before expiry
const MAX_REFRESH_ATTEMPTS = 3;
const TOKEN_VERIFY_INTERVAL = 30000; // 30 seconds

let isRefreshing = false;
let refreshSubscribers = [];
let globalAuth = null; // Store auth instance

export function provideAuth() {
  const router = useRouter();

  const state = reactive({
    isLoggedIn: false,
    isLoading: true, // Start with loading state
    username: "",
    accessToken: "",
    refreshToken: "",
    refreshAttempts: 0,
    lastRefreshTime: null,
    authorities: [],
    role: "",
    email_confirmed: false,
  });

  // Initialize auth state immediately
  setTimeout(() => {
    checkAuthStatus();
  }, 0);

  let refreshTimer = null;
  let verificationTimer = null;
  let visibilityHandler = null;

  // Enhanced token verification with retry logic
  const verifyAndRefreshToken = async () => {
    if (typeof window === "undefined") return;

    const accessToken = localStorage.getItem("accessToken");

    if (!accessToken) {
      return false;
    }

    try {
      const decoded = jwtDecode(accessToken);
      const now = Math.floor(Date.now() / 1000);

      // Check if token needs refresh (within margin)
      if (decoded.exp - now < REFRESH_MARGIN) {
        return await refreshAccessToken();
      }

      return true;
    } catch (error) {
      console.warn("Token validation failed:", error);
      return await refreshAccessToken();
    }
  };

  async function refreshAccessToken() {
    if (state.refreshAttempts >= MAX_REFRESH_ATTEMPTS) {
      logout();
      return false;
    }

    if (isRefreshing) {
      return new Promise((resolve) => {
        refreshSubscribers.push(resolve);
      });
    }

    isRefreshing = true;

    try {
      const refreshToken = localStorage.getItem("refreshToken");
      if (!refreshToken) {
        logout();
        return false;
      }

      const response = await api.post("/auth/refresh", {
        refresh_token: refreshToken,
      });

      if (response.data.access_token) {
        state.accessToken = response.data.access_token;
        localStorage.setItem("accessToken", response.data.access_token);

        if (response.data.refresh_token) {
          state.refreshToken = response.data.refresh_token;
          localStorage.setItem("refreshToken", response.data.refresh_token);
        }

        state.refreshAttempts = 0;
        state.lastRefreshTime = Date.now();

        refreshSubscribers.forEach((callback) => callback(true));
        refreshSubscribers = [];

        const decoded = jwtDecode(response.data.access_token);
        state.username = decoded.username; // Update reactive state
        state.authorities = decoded.authorities || []; // Update reactive state
        state.role = decoded.role || ""; // Update reactive state
        state.email_confirmed = decoded.email_confirmed || false; // Update reactive state
        localStorage.setItem("username", decoded.username); // Keep localStorage consistent
        scheduleTokenRefresh(decoded.exp);
        return true;
      }
    } catch (error) {
      state.refreshAttempts++;
      console.error(
        `Token refresh failed (attempt ${state.refreshAttempts}):`,
        error
      );

      refreshSubscribers.forEach((callback) => callback(false));
      refreshSubscribers = [];

      if (state.refreshAttempts >= MAX_REFRESH_ATTEMPTS) {
        logout();
        return false;
      }

      return false;
    } finally {
      isRefreshing = false;
    }

    return false;
  }

  async function logout() {
    // Attempt to call the backend logout endpoint
    // The Authorization header will be added by the request interceptor
    performBackendLogout()
      .then(() => {
        console.log("Backend logout successful");
      })
      .catch((error) => {
        // Error is expected if the token was already invalid or session expired.
        // Client-side cleanup should still occur.
        console.error(
          "Backend logout failed. Proceeding with client-side logout.",
          error
        );
      });

    // Perform all client-side cleanup
    localStorage.removeItem("accessToken");
    localStorage.removeItem("refreshToken");
    localStorage.removeItem("username");

    state.isLoggedIn = false;
    state.username = "";
    state.accessToken = "";
    state.refreshToken = "";
    state.refreshAttempts = 0;
    state.lastRefreshTime = null;
    state.authorities = [];
    state.role = "";
    state.email_confirmed = false;

    if (refreshTimer) {
      clearTimeout(refreshTimer);
      refreshTimer = null;
    }

    if (verificationTimer) {
      clearInterval(verificationTimer);
      verificationTimer = null;
    }
    if (visibilityHandler) {
      document.removeEventListener("visibilitychange", visibilityHandler);
      visibilityHandler = null;
    }

    isRefreshing = false;
    refreshSubscribers = [];

    console.log("Client-side logout completed.");

    // Navigate to login page
    if (router) {
      router.push("/login");
    } else {
      console.warn("Router instance not available in logout function.");
      // Fallback if router is not available for some reason
      // window.location.pathname = '/login';
    }
  }

  function scheduleTokenRefresh(expiryTime) {
    if (refreshTimer) {
      clearTimeout(refreshTimer);
    }

    const now = Math.floor(Date.now() / 1000);
    const timeUntilRefresh = Math.max(0, expiryTime - REFRESH_MARGIN - now);
    console.log("Scheduling token refresh in", timeUntilRefresh, "seconds");

    refreshTimer = setTimeout(refreshAccessToken, timeUntilRefresh * 1000);
  }

  // Setup continuous token verification
  const startTokenVerification = () => {
    if (verificationTimer) {
      clearInterval(verificationTimer);
    }

    verificationTimer = setInterval(async () => {
      console.log("Token verification check at", new Date().toISOString());
      const isValid = await verifyAndRefreshToken();
      if (!isValid && state.isLoggedIn) {
        console.warn("Token invalid during verification check, logging out");
        logout();
      }
    }, TOKEN_VERIFY_INTERVAL);

    visibilityHandler = () => {
      if (document.visibilityState === "visible") {
        console.log("Tab became visible, triggering immediate token check");
        verifyAndRefreshToken();
      }
    };
    document.addEventListener("visibilitychange", visibilityHandler);
  };

  function login(accessToken, refreshToken, username) {
    localStorage.setItem("accessToken", accessToken);
    localStorage.setItem("refreshToken", refreshToken);
    localStorage.setItem("username", username);

    state.isLoggedIn = true;
    state.username = username;
    state.accessToken = accessToken;
    state.refreshToken = refreshToken;
    state.refreshAttempts = 0;

    const decoded = parseToken(accessToken);
    if (decoded) {
      state.authorities = decoded.authorities || [];
      state.role = decoded.role || "";
      state.email_confirmed = decoded.email_confirmed || false;
      scheduleTokenRefresh(decoded.exp);
    } else {
      state.authorities = [];
      state.role = "";
      state.email_confirmed = false;
    }

    startTokenVerification();
    void mergeAnonymousProgressThenClear();
  }

  async function mergeAnonymousProgressThenClear() {
    try {
      const payloads = getAllProgressForMerge();
      for (const p of payloads) {
        await mergeProgress({ collection_id: p.collection_id, level_progress: p.level_progress });
      }
      if (payloads.length) clearAfterMerge();
    } catch (err) {
      console.warn("Merge anonymous progress failed:", err);
    }
  }

  function parseToken(token) {
    try {
      return jwtDecode(token);
    } catch (error) {
      console.error("Failed to decode token:", error);
      return null;
    }
  }

  async function checkAuthStatus() {
    if (typeof window === "undefined") return;

    state.isLoading = true;
    try {
      const accessToken = localStorage.getItem("accessToken");
      const refreshToken = localStorage.getItem("refreshToken");

      if (!accessToken || !refreshToken) {
        state.isLoggedIn = false;
        return false;
      }

      const decoded = parseToken(accessToken);
      if (!decoded) {
        state.isLoggedIn = false;
        return false;
      }

      const now = Math.floor(Date.now() / 1000);
      if (now >= decoded.exp) {
        if (refreshToken) {
          state.refreshToken = refreshToken;
          const refreshed = await refreshAccessToken();
          state.isLoggedIn = refreshed;
          return refreshed;
        } else {
          logout();
          return false;
        }
      } else {
        state.isLoggedIn = true;
        state.username = decoded.username;
        state.authorities = decoded.authorities || [];
        state.role = decoded.role || "";
        state.email_confirmed = decoded.email_confirmed || false;
        state.accessToken = accessToken;
        state.refreshToken = refreshToken;

        scheduleTokenRefresh(decoded.exp);
        startTokenVerification();
        return true;
      }
    } finally {
      state.isLoading = false;
    }
  }

  const auth = {
    state,
    login,
    logout,
    checkAuthStatus,
    refreshAccessToken,
  };

  setAuthInstance(auth);

  provide(authKey, auth);
  globalAuth = auth; // Store the auth instance
  return auth;
}

export function useAuth() {
  const auth = inject(authKey);
  if (!auth) {
    throw new Error("useAuth() called without provider");
  }
  return auth;
}
