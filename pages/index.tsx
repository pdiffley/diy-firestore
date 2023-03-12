import "@code-hike/mdx/dist/index.css";
import type { AppProps } from "next/app";

export default function Index({ Component, pageProps }: AppProps) {
  return (
    <div>
      <h1>Index</h1>
      <a href="/posts/01-intro" target="_blank">
        01-intro
      </a>{" "}
      <br></br>
      <a href="/posts/02-defining-requirements" target="_blank">
        02-defining-requirements
      </a>{" "}
      <br></br>
      <a href="/posts/03-the-basic-database" target="_blank">
        {" "}
        03-the-basic-database{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/04-basic-operations" target="_blank">
        {" "}
        04-basic-operations{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/05-basic-subscriptions" target="_blank">
        {" "}
        05-basic-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/06-queries-first-attempt" target="_blank">
        {" "}
        06-queries-first-attempt{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/07-custom-comparator" target="_blank">
        {" "}
        07-custom-comparator{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/08-simple-queries-take-two" target="_blank">
        {" "}
        08-simple-queries-take-two{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/09-simple_query-subscriptions" target="_blank">
        {" "}
        09-simple_query-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/10-composite-queries" target="_blank">
        {" "}
        10-composite-queries{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/11-composite-subscriptions" target="_blank">
        {" "}
        11-composite-subscriptions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/12-subscription-update-queues" target="_blank">
        {" "}
        12-subscription-update-queues{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/13-client-connection-server" target="_blank">
        {" "}
        13-client-connection-server{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/14-transactions" target="_blank">
        {" "}
        14-transactions{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/15-security-rules" target="_blank">
        {" "}
        15-security-rules{" "}
      </a>{" "}
      <br></br>
      <a href="/posts/16-bonus-features-and-configurability" target="_blank">
        {" "}
        16-bonus-features-and-configurability{" "}
      </a>{" "}
      <br></br>
    </div>
  );
}
