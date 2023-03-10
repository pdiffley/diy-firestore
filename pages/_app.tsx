import "@code-hike/mdx/dist/index.css";
import "../styles/globals.css";
import type { AppProps } from "next/app";

export default function DiyFirestoreApp({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />;
}
