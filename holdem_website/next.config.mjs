/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  async redirects() {
    return [
      {
        source: '/vs/dropoint',
        destination: '/vs/droppoint',
        permanent: true,
      },
      {
        source: '/vs/dropoint/:path*',
        destination: '/vs/droppoint/:path*',
        permanent: true,
      },
      {
        source: '/vs/bucket',
        destination: '/vs',
        permanent: true,
      },
    ]
  },
}

export default nextConfig
