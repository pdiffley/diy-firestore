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
import * as matter from "gray-matter";

export default function Post({ source, title, postsData }) {
  return (
    <>
      <Head>
        <title>DIY Firestore: {title}</title>
      </Head>
      <main>
        <div
          style={{
            display: "flex",
            flexDirection: "row",
            justifyContent: "space-around",
          }}
        >
          <nav style={{ alignItems: "flex-start" }}>
            <div>
              <Link href="/">Home</Link>
            </div>
            {postsData.map(({ title, slug }) => (
              <div key={slug}>
                <Link href={`/posts/${slug}`}>{title}</Link>
              </div>
            ))}
          </nav>
          <article>
            <MDXRemote {...source} components={{ CH, Image }} />
          </article>
        </div>
      </main>
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
  const postsData = filenames
    .map((x) => {
      const contents = fs.readFileSync(path.join(process.cwd(), "posts", x), {
        encoding: "utf8",
      });
      return {
        ...matter(contents).data,
        slug: x.slice(3, x.length - 4),
      };
    })
    .sort(({ index: a }, { index: b }) => a - b);

  const { content, data } = matter(source);
  const mdxSource = await serialize(content, {
    mdxOptions: {
      remarkPlugins: [[remarkCodeHike, { autoImport: false, theme }]],
      useDynamicImport: true,
    },
  });
  return { props: { source: mdxSource, ...data, postsData } };
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
