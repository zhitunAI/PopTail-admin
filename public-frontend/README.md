# Public Frontend

Next + Tailwind CSS public-facing site for `PopTail-admin`.

This site is part of a project built on top of the `vue-vben-admin` framework:

- [https://github.com/vbenjs/vue-vben-admin](https://github.com/vbenjs/vue-vben-admin)

The home page now follows a Jimeng-style AI creation workspace: left rail,
centered Agent headline, large prompt panel, tool cards, inspiration tabs and a
content gallery.

## Stack

- Next.js
- Tailwind CSS
- shadcn/ui-style local components based on Radix Slot, CVA, clsx and tailwind-merge
- lucide-react icons

shadcn/ui was selected because its official site and GitHub repository currently show about 113k stars, and it fits the requested Next + Tailwind direction.

## Run

```bash
pnpm install
pnpm dev
```

Default URL:

```text
http://127.0.0.1:3000
```

## Interaction

- Home page top-right user entry opens an in-page member panel.
- The standalone console route has been removed; account basics live on the home page.
- Public settings are read from `http://127.0.0.1:8888/public/frontend/settings` when available.
