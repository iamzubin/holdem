import createMDX from '@next/mdx'

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  pageExtensions: ['js', 'jsx', 'ts', 'tsx', 'md', 'mdx'],
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
    ];
  },
};

const withMDX = createMDX({
  extension: /\.mdx?$/,
})

export default withMDX(nextConfig)
