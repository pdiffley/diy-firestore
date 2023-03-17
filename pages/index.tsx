import fs from "fs";
import path from "path";
import * as matter from "gray-matter";
import Link from "next/link";

import "@code-hike/mdx/dist/index.css";
import { main, blogSummary, header, nav } from "../styles/index.module.css";

export default function Index({
  postsData,
}: {
  postsData: Array<{ title: string; slug: string; subtitle: string }>;
}) {
  return (
    <main className={main}>
      <header className={header}>
        <h1>DIY Firestore</h1>
        <p className={blogSummary}>
          A series of blog posts about building a Firestore-like database from
          scratch. How hard could it be?
        </p>
        <p>by Phillip Diffley</p>
      </header>
      <nav className={nav}>
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
        ...matter(contents).data,
        slug: x.slice(3, x.length - 4),
      };
    })
    .sort(({ index: a }, { index: b }) => a - b);

  return { props: { postsData } };
}
