import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";

const Footer = () => (
  <footer className="border-t border-border/80 py-8">
    <Container className="flex flex-col gap-4 text-body-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
      <BrandWordmark className="text-xs" />
      <p>Design Foundation · Branch 01</p>
    </Container>
  </footer>
);

export { Footer };
