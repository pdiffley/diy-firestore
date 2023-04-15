import fs from "fs";
import path from "path";
import Head from "next/head";
import Link from "next/link";
import Image from "next/image";
import { serialize } from "next-mdx-remote/serialize";
import { MDXRemote } from "next-mdx-remote";
import { remarkCodeHike } from "@code-hike/mdx";
import { CH } from "@code-hike/mdx/components";
import theme from "shiki/themes/dracula-soft.json";
import matter from "gray-matter";

import { Post } from "../../model/Post";
import styles from "../../styles/post.module.css";

const { content, homeLink, nav, article, footer } = styles;

type Props = {
  source: Awaited<ReturnType<typeof serialize>>;
  title: string;
  previousPost: Post;
  nextPost: Post;
};

export default function PostPage({
  source,
  title,
  previousPost,
  nextPost,
}: Props) {
  return (
    <>
      <Head>
        <title>DIY Firestore: {title}</title>
        <meta name="robots" content="noindex" />
      </Head>
      <div className={content}>
        <nav className={nav}>
          <div className={homeLink}>
            <Link href="/">DIY Firestore</Link>
          </div>
          {/* <div>
            {previousPost && (
              <Link href={`/posts/${previousPost.slug}`}>
                &larr; {previousPost.title}
              </Link>
            )}
            {previousPost && nextPost && " | "}
            {nextPost && (
              <Link href={`/posts/${nextPost.slug}`}>
                {nextPost.title} &rarr;
              </Link>
            )}
          </div> */}
        </nav>
        <main id="main-content">
          <article className={article}>
            <MDXRemote {...source} components={{ CH, Image }} />
          </article>
        </main>
        <footer className={footer}>
          <div>
            {previousPost && (
              <Link href={`/posts/${previousPost.slug}`}>
                &larr; {previousPost.title}
              </Link>
            )}
            {previousPost && nextPost && " | "}
            {nextPost && (
              <Link href={`/posts/${nextPost.slug}`}>
                {nextPost.title} &rarr;
              </Link>
            )}
          </div>
        </footer>
      </div>
    </>
  );
}

export async function getStaticProps({ params }) {
  const slug = params.slug;
  const filenames = fs.readdirSync(path.join(process.cwd(), "posts"));
  const filename = filenames.find((f) => f.includes(slug));
  const source = fs.readFileSync(path.join(process.cwd(), "posts", filename), {
    encoding: "utf8",
  });
  const postsData = filenames.map((x) => {
    const contents = fs.readFileSync(path.join(process.cwd(), "posts", x), {
      encoding: "utf8",
    });
    return {
      ...(matter(contents).data as Post),
      slug: x.slice(3, x.length - 4),
    };
  });

  const { content, data } = matter(source);
  const mdxSource = await serialize(content, {
    mdxOptions: {
      remarkPlugins: [[remarkCodeHike, { autoImport: false, theme }]],
      useDynamicImport: true,
    },
  });
  const previousPost =
    postsData.find(({ index }) => index === data.index - 1) ?? null;
  const nextPost =
    postsData.find(({ index }) => index === data.index + 1) ?? null;
  return { props: { source: mdxSource, ...data, previousPost, nextPost } };
}

export function getStaticPaths() {
  const posts = fs
    .readdirSync(path.join(process.cwd(), "posts"))
    .map((page) => page.slice(3, page.length - 4));
  return {
    paths: posts.map((slug) => ({ params: { slug: [slug] } })),
    fallback: false,
  };
}
