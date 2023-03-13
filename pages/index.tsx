import "@code-hike/mdx/dist/index.css";
import type { AppProps } from "next/app";

export default function Index({ Component, pageProps }: AppProps) {
  return (
    <div>
      <h1>Index</h1>
      <a href="/posts/intro" target="_blank">
        01-intro
      </a>{" "}
      <br></br>
      <a href="/posts/defining-requirements" target="_blank">
        02-defining-requirements
      </a>{" "}
      <br></br>
      <a href="/posts/the-basic-database" target="_blank">
        {" "}
        03-the-basic-database{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/basic-operations" target="_blank">
        {" "}
        04-basic-operations{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/transactions" target="_blank">
        {" "}
        05-transactions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/basic-subscriptions" target="_blank">
        {" "}
        06-basic-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/queries-first-attempt" target="_blank">
        {" "}
        07-queries-first-attempt{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/custom-comparator" target="_blank">
        {" "}
        08-custom-comparator{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/simple-queries-take-two" target="_blank">
        {" "}
        09-simple-queries-take-two{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/simple_query-subscriptions" target="_blank">
        {" "}
        10-simple_query-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/composite-queries" target="_blank">
        {" "}
        11-composite-queries{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/composite-subscriptions" target="_blank">
        {" "}
        12-composite-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/subscription-update-queues" target="_blank">
        {" "}
        13-subscription-update-queues{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/client-connection-server" target="_blank">
        {" "}
        14-client-connection-server{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/security-rules" target="_blank">
        {" "}
        15-security-rules{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/bonus-features-and-configurability" target="_blank">
        {" "}
        16-bonus-features-and-configurability{" "}
      </a>{" "}
      <br></br>
    </div>
  );
}
