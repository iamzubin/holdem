import type { MetadataRoute } from 'next'
import { WEBSITE_URL } from '@/lib/constants'

export default function sitemap(): MetadataRoute.Sitemap {
  const routes = [
    '',
    '/download',
    '/vs',
    '/vs/dropshelf',
    '/vs/droppoint',
    '/vs/dropover',
    '/vs/yoink',
    '/pricing',
    '/changelog',
    '/docs',
  ]

  return routes.map((route) => ({
    url: `${WEBSITE_URL}${route || '/'}`,
    lastModified: new Date(),
    changeFrequency: route === '' ? 'weekly' : 'monthly',
    priority: route === '' ? 1 : route.startsWith('/vs/') ? 0.8 : 0.7,
  }))
}
