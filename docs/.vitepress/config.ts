import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Chain Forge",
  description: "A Foundry-inspired multi-chain development tool suite for local blockchain development",
  base: '/chain-forge/',
  head: [['link', { rel: 'icon', href: '/chain-forge/logo.ico' }]],

  ignoreDeadLinks: [
    /^http:\/\/localhost/,
  ],

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/chain-forge/logo.ico',

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'API', link: '/api/overview' },
      { text: 'Examples', link: '/examples/typescript' }
    ],

    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'What is Chain Forge?', link: '/guide/what-is-chain-forge' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Installation', link: '/guide/installation' }
        ]
      },
      {
        text: 'Solana',
        items: [
          { text: 'Overview', link: '/solana/overview' },
          { text: 'CLI Commands', link: '/solana/cli' },
          { text: 'Configuration', link: '/solana/configuration' },
          { text: 'Account Management', link: '/solana/accounts' },
          { text: 'Program Deployment', link: '/solana/program-deployment' }
        ]
      },
      {
        text: 'TypeScript Package',
        items: [
          { text: 'Installation', link: '/typescript/installation' },
          { text: 'Basic Usage', link: '/typescript/basic-usage' },
          { text: 'API Reference', link: '/api/overview' }
        ]
      },
      {
        text: 'Examples',
        items: [
          { text: 'TypeScript Examples', link: '/examples/typescript' },
          { text: 'Program Deployment', link: '/examples/program-deployment' },
          { text: 'CLI Workflows', link: '/examples/cli-workflows' }
        ]
      },
      {
        text: 'Contributing',
        items: [
          { text: 'Development Guide', link: '/contributing/development' },
          { text: 'Architecture', link: '/contributing/architecture' },
          { text: 'Testing', link: '/contributing/testing' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/christopherferreira9/chain-forge' }
    ],

    footer: {
      message: 'Released under the MIT and Apache-2.0 Licenses.',
      copyright: 'Copyright © 2024-present Chain Forge Contributors'
    },

    search: {
      provider: 'local'
    }
  }
})
