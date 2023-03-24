/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  trailingSlash: true,
  basePath: "/diy-firestore",
  images: {
    unoptimized: true,
  },
};

module.exports = nextConfig;
