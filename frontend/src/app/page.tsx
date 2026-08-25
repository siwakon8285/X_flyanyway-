import { Hero } from "@/components/home/Hero";
import { Container } from "@/components/layout/Container";

const Home = () => (
  <>
    <Hero />
    <section
      aria-labelledby="journey-heading"
      className="flex min-h-[65svh] items-center border-t border-border/80 bg-background py-section-md"
      id="journey"
    >
      <Container>
        <p className="text-label text-brand">Beyond the horizon</p>
        <h2 className="mt-4 max-w-4xl text-h1 text-balance" id="journey-heading">
          The journey continues.
        </h2>
        <p className="mt-6 max-w-reading text-body-lg text-muted-foreground">
          The next chapter of X-Fly is taking shape.
        </p>
      </Container>
    </section>
  </>
);

export default Home;
