import fs from "fs";
import path from "path";
import matter from "gray-matter";
import Link from "next/link";

import "@code-hike/mdx/dist/index.css";

import { Post } from "../model/Post";
import styles from "../styles/index.module.css";
import { Helmet } from "react-helmet";

const { main, blogSummary, header } = styles;

export default function Index({ postsData }: { postsData: Array<Post> }) {
  return (
    <main className={main}>
      <Helmet>
        <meta name="robots" content="noindex" />
      </Helmet>

      <header className={header}>
        <h1>DIY Firestore</h1>

        <p className={blogSummary}>
          A series of blog posts about building a Firestore-like database from
          scratch.
        </p>
        <p>by Phillip Diffley</p>
      </header>
      <nav>
        {postsData.map(({ title, slug, subtitle }) => (
          <div key={slug}>
            <h2>
              <Link href={`/posts/${slug}`}>{title}</Link>
            </h2>
            <p>{subtitle}</p>
          </div>
        ))}
      </nav>
    </main>
  );
}

export async function getStaticProps() {
  const filenames = fs.readdirSync(path.join(process.cwd(), "posts"));
  const postsData = filenames
    .map((x) => {
      const contents = fs.readFileSync(path.join(process.cwd(), "posts", x), {
        encoding: "utf8",
      });
      return {
        ...(matter(contents).data as Post),
        slug: x.slice(3, x.length - 4),
      };
    })
    .sort(({ index: a }, { index: b }) => a - b);

  return { props: { postsData } };
}
