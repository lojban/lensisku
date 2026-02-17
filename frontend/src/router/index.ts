import type { RouteRecordRaw } from "vue-router";

import { checkAuthStateForGuard } from "../composables/useAuthGuard";

import {
  supportedLocales,
  defaultLocale,
  type SupportedLocale,
} from "../config/locales";
export type { SupportedLocale } from "../config/locales";

// Create base routes without locale prefix
// The NotFound route MUST be the last one in this array.
const baseRoutes: Array<RouteRecordRaw> = [
  {
    path: "",
    name: "Home",
    component: () => import("../pages/HomePage.vue"),
    props: (route) => ({
      urlSearchQuery: route.query.q,
      urlSearchMode: route.query.mode || "dictionary",
    }),
  },
  {
    path: "/sutra_sisku",
    name: "FastSearch",
    component: () => import("../pages/FastSearchPage.vue"),
  },
  {
    path: "/message/:id",
    name: "MessageDetail",
    component: () => import("../pages/MessageDetail.vue"),
    props: (route) => ({
      id: Number(route.params.id) || undefined,
      searchTerm: route.query.highlight?.toString() || ''
    }),
  },
  {
    path: "/thread/:subject",
    name: "ThreadView",
    component: () => import("../pages/ThreadView.vue"),
    props: (route) => ({
      subject: decodeURIComponent(route.params.subject as string),
      searchTerm: route.query.highlight,
    }),
  },
  {
    path: "/tiktoknu",
    name: "Tiktoknu",
    component: () => import("../pages/TiktoknuPage.vue"),
  },
  {
    path: "/signup",
    name: "Signup",
    component: () => import("../pages/SignupPage.vue"),
  },
  {
    path: "/login",
    name: "Login",
    component: () => import("../pages/LoginPage.vue"),
  },
  {
    path: "/profile",
    name: "Profile",
    component: () => import("../pages/ProfilePage.vue"),
  },
  {
    path: "/valsi/add",
    name: "AddDefinition",
    component: () => import("../pages/UpsertDefinition.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/definition/markdown",
    name: "UpsertDefinitionMarkdownNew",
    component: () => import("../pages/UpsertDefinitionMarkdown.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/definition/markdown/:id",
    name: "UpsertDefinitionMarkdownEdit",
    component: () => import("../pages/UpsertDefinitionMarkdown.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/valsi/:id",
    name: "Entry",
    component: () => import("../pages/EntryPage.vue"),
  },
  {
    path: "/definition/:id/edit",
    name: "EditDefinition",
    component: () => import("../pages/UpsertDefinition.vue"),
    meta: { requiresAuth: true },
    props: (route) => ({ id: route.params.id }),
  },
  {
    path: "/definition_link/:id/discussion",
    name: "DefinitionLinkDiscussion",
    component: () => import("../pages/DefinitionLinkDiscussion.vue"),
    props: (route) => ({ id: parseInt(route.params.id as string) }),
  },
  {
    path: "/reset-password",
    name: "ResetPassword",
    component: () => import("../pages/PasswordReset.vue"),
  },
  {
    path: "/comments",
    name: "CommentList",
    component: () => import("../pages/CommentList.vue"),
    props: (route) => ({
      valsiId: parseInt(route.query.valsi_id as string) || 0,
      natlangWordId: parseInt(route.query.natlang_word_id as string) || 0,
      definitionId: parseInt(route.query.definition_id as string) || 0,
      commentId: parseInt(route.query.comment_id as string) || 0,
      scrollTo: parseInt(route.query.scroll_to as string) || 0,
      threadId: parseInt(route.query.thread_id as string) || 0,
    }),
  },
  {
    path: "/comments/thread",
    name: "comments-thread",
    component: () => import("../pages/CommentList.vue"),
    props: (route) => ({
      valsiId: parseInt(route.query.valsi_id as string) || 0,
      natlangWordId: parseInt(route.query.natlang_word_id as string) || 0,
      definitionId: parseInt(route.query.definition_id as string) || 0,
      commentId: parseInt(route.query.comment_id as string) || 0,
    }),
  },
  {
    path: "/comments/new-thread",
    name: "NewThread",
    component: () => import("../pages/NewThreadPage.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/recent",
    name: "RecentChanges",
    component: () => import("../pages/RecentChanges.vue"),
  },
  {
    path: "/definition/:id/history",
    name: "VersionHistory",
    component: () => import("../pages/VersionHistory.vue"),
    props: true,
    meta: { requiresAuth: true },
  },
  {
    path: "/user/:username",
    name: "UserProfile",
    component: () => import("../pages/ProfilePage.vue"),
    props: true,
  }, // Note: Same component as /profile
  {
    path: "/users",
    name: "UserList",
    component: () => import("../pages/UserList.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/languages",
    name: "LanguageList",
    component: () => import("../pages/LanguageList.vue"),
  },
  {
    path: "/export",
    name: "DictionaryExport",
    component: () => import("../pages/DictionaryExport.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/definitions/export-pairs",
    name: "ExportPairs",
    component: () => import("../pages/ExportPairs.vue"),
  },
  {
    path: "/export/cached",
    name: "CachedExports",
    component: () => import("../pages/CachedExports.vue"),
  },
  {
    path: "/collections",
    name: "CollectionList",
    component: () => import("../pages/CollectionList.vue"),
  },
  {
    path: "/collections/:id",
    name: "CollectionDetail",
    component: () => import("../pages/CollectionDetail.vue"),
    props: (route) => ({
      collectionId: route.params.id,
    }),
  },
  {
    path: "/reactions",
    name: "Reactions",
    component: () => import("../pages/ReactionsPage.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/user/:username/activity",
    name: "UserActivity",
    component: () => import("../pages/UserContributions.vue"),
    props: true,
  },
  {
    path: "/confirm-email",
    name: "ConfirmEmail",
    component: () => import("../pages/EmailConfirmation.vue"),
  },
  {
    path: "/collections/:collectionId/flashcards",
    name: "FlashcardCollection",
    component: () => import("../pages/FlashcardCollectionView.vue"),
    props: (route) => ({
      collectionId: route.params.collectionId,
    }),
    meta: { requiresAuth: true },
  },
  {
    path: "/collections/:collectionId/flashcards/study",
    name: "FlashcardStudy",
    component: () => import("../pages/FlashcardStudyView.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/collections/:collectionId/levels",
    name: "FlashcardLevels",
    component: () => import("../pages/FlashcardLevels.vue"),
    props: true,
  },
  {
    path: "/change-password",
    name: "ChangePassword",
    component: () => import("../pages/ChangePassword.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/paypal/return",
    name: "PaypalReturn",
    component: () => import("../pages/PaypalReturn.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/balance",
    name: "Balance",
    component: () => import("../pages/PaymentPage.vue"),
    meta: { requiresAuth: true },
  },
  {
    path: "/bulk-import",
    name: "BulkImport",
    component: () => import("../pages/BulkImportDefinitions.vue"),
    meta: {
      requiresAuth: true,
      requiredPermissions: ["bulk_import"],
    },
  },
  {
    path: "/bulk-import/clients",
    name: "BulkImportClients",
    component: () => import("../pages/BulkImportClients.vue"),
    meta: {
      requiresAuth: true,
      requiredPermissions: ["bulk_import"],
    },
  },
  {
    path: "/ko-zehe-sarji",
    name: "AprilFools",
    component: () => import("../pages/AprilFools.vue"),
  },
  // NotFound route must be last among base routes to catch all unmatched paths within a locale
  {
    path: "/:pathMatch(.*)*",
    name: "NotFound",
    component: () => import("../pages/NotFound.vue")
  },
];

// Generate locale-specific routes
const localeRoutes = supportedLocales.flatMap((locale) =>
  baseRoutes.map((route) => {
    if (route.name?.toString().endsWith(`-${locale}`)) {
      throw new Error(`Route name conflict: ${route.name as string}`);
    }
    return {
      ...route,
      path: `/${locale}${route.path}`, // This will correctly create /en/:pathMatch(.*)* for NotFound
      name: `${String(route.name)}-${locale}`,
      props: route.props,
    };
  })
) as Array<RouteRecordRaw>;

export const setupRouterGuards = (router: any, isClient: boolean) => {
  router.beforeEach(async (to: any, from: any, next: any) => {
    const baseToName = to.name?.split("-")[0];

    if (to.query.redirect_loop) {
      if (baseToName === 'Login') {
        // Already in a redirect loop trying to reach Login.
        // Navigate to Login but clear the flag to break the loop.
        const newQuery = { ...to.query };
        delete newQuery.redirect_loop;
        return next({ name: to.name, params: to.params, query: newQuery, replace: true });
      }
      // For other routes with redirect_loop, go to root.
      return next('/');
    }
    
    if (isClient) {
      const authRoutes = ["Login", "Signup", "ResetPassword", "ConfirmEmail"];
      const isNotFoundRoute = baseToName === 'NotFound';
      const routeRequiresAuth = to.matched.some((record: any) => record.meta.requiresAuth);

      const isAuthenticated = await checkAuthStateForGuard();
      if (
        !isAuthenticated &&
        baseToName &&
        !authRoutes.includes(baseToName) &&
        !isNotFoundRoute && // Do not redirect from NotFound page
        routeRequiresAuth // Only redirect if route requires auth
      ) {
        sessionStorage.setItem("redirectPath", to.fullPath);

        // Extract locale from the 'to' route's path
        const pathSegments = to.path.split('/');
        let targetLocale: SupportedLocale = defaultLocale; // Default if extraction fails or not a locale path
        
        // pathSegments[0] is an empty string due to the leading slash (e.g., "/ru/profile" -> ["", "ru", "profile"])
        if (pathSegments.length > 1 && supportedLocales.includes(pathSegments[1] as SupportedLocale)) {
          targetLocale = pathSegments[1] as SupportedLocale;
        }
        
        return next({
          name: `Login-${targetLocale}`, // Use the extracted locale for the Login route name
          query: { redirect_loop: from.name ? '1' : undefined }
        });
      }
    }
    next();
  });
};

// Export the routes array with a root redirect and locale-specific routes
export const routes: Array<RouteRecordRaw> = [
  {
    // Redirect root to default locale
    path: "/",
    redirect: () => {
      const storedLocale =
        typeof window !== "undefined"
          ? (localStorage.getItem("selectedLocale") as SupportedLocale | null)
          : null;
      const preferredLocale =
        storedLocale && supportedLocales.includes(storedLocale)
          ? storedLocale
          : defaultLocale;
      return `/${preferredLocale}`;
    },
  },
  ...localeRoutes, // Locale-specific routes, including localized NotFound (e.g., /en/:pathMatch(.*)*)
  {
    // This is the VERY LAST route.
    // It catches paths that were not matched by localeRoutes (e.g., /foo/bar, or /unsupportedlocale/baz)
    // and redirects them to a path prefixed with the preferred/default locale.
    // This ensures any valid-looking path eventually gets a locale prefix and is re-evaluated.
    // If it's still not found after prefixing, the NotFound-<locale> route from localeRoutes will handle it.
    path: "/:pathMatch(.*)*",
    redirect: (to) => {
      const path = to.path.replace(/^\/+/g, ''); // Full path without leading slash

      const storedLocale =
        typeof window !== "undefined"
          ? (localStorage.getItem("selectedLocale") as SupportedLocale | null)
          : null;
      const preferredLocale =
        storedLocale && supportedLocales.includes(storedLocale)
          ? storedLocale
          : defaultLocale; // Use defaultLocale as a reliable fallback

      return `/${preferredLocale}/${path}`;
    },
  },
];
