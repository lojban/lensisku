import { ViteSSG } from 'vite-ssg'
import Applic from './App.vue'
import { routes, setupRouterGuards } from './router'
import i18n from './i18n'
import './style.css'
import '@milkdown/crepe'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'

export const createApp = ViteSSG(
  Applic,
  { routes },
  ({ app, router, isClient }) => {
    setupRouterGuards(router, isClient)

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

      const loadMathJax = () => {
        const script = document.createElement('script')
        script.src = 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js'
        script.async = true
        document.head.appendChild(script)
      }
      loadMathJax()
    }
  },
  {}
)
