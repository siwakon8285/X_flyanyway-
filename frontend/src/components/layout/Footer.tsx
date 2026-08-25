import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";

const Footer = () => (
  <footer className="border-t border-border/80 py-8">
    <Container className="flex flex-col gap-4 text-body-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
      <BrandWordmark className="text-xs" />
      <p>Design + Motion Foundation · Branch 02</p>
    </Container>
  </footer>
);

export { Footer };
