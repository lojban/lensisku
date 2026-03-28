import { ViteSSG } from 'vite-ssg'
import Applic from './App.vue'
import { routes, setupRouterGuards } from './router' // Import routes and guard setup
import i18n from './i18n' // Import the i18n instance
import './style.css' // Keep global styles if any
import '@milkdown/crepe' // Eagerly import Crepe
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'

// `export const createApp` is required for vite-ssg
export const createApp = ViteSSG(
  // the root component
  Applic,
  // vue-router options with routes array
  { routes }, // Pass routes in the RouterOptions structure
  // function to configure the app instance
  ({ app, router, isClient }) => {
    // Setup navigation guards
    setupRouterGuards(router, isClient)

    // Install i18n instance
    app.use(i18n)

    if (isClient) {
      const w = window as any
      w.MathJax = {
        tex: {
          inlineMath: [
            ['$', '$'],
            ['\\(', '\\)'],
          ],
          displayMath: [
            ['$$', '$$'],
            ['\\[', '\\]'],
          ],
          processEscapes: true,
        },
        options: {
          // Include `code` so inline markdown `<code>$x_1$</code>` stays literal (matches jbovlaste).
          skipHtmlTags: ['script', 'noscript', 'style', 'textarea', 'pre', 'code'],
        },
      }

      // Load MathJax
      const loadMathJax = () => {
        const script = document.createElement('script')
        script.src = 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js'
        script.async = true
        document.head.appendChild(script)
      }
      loadMathJax()
    }

    // Potentially handle initial state hydration here if needed
    // if (isClient && initialState) {
    //   // Hydrate state...
    // }
  },
  // SSG Options (optional)
  {
    // Specify routes to pre-render (defaults to routes defined in router)
    // routes: ['/', '/about'], // Example if you had an /about page
    // Add base path if deploying to a subdirectory
    // base: '/my-app/',
  }
)
