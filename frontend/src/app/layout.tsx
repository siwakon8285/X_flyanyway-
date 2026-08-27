import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import { SiteShell } from "@/components/layout/SiteShell";
import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";
import { INITIAL_HASH_BOOTSTRAP_SCRIPT } from "@/lib/motion/initialHash";

import "lenis/dist/lenis.css";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "X-Fly Anyway · Go Anywhere. Fly Different.",
  description:
    "A premium global aviation experience designed to take you anywhere.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
      suppressHydrationWarning
    >
      <head>
        <script
          dangerouslySetInnerHTML={{ __html: INITIAL_HASH_BOOTSTRAP_SCRIPT }}
          id="initial-hash-bootstrap"
        />
      </head>
      <body className="flex min-h-full flex-col">
        <SmoothScrollProvider>
          <SiteShell>{children}</SiteShell>
        </SmoothScrollProvider>
      </body>
    </html>
  );
}
