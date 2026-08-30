import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { cookies } from "next/headers";

import { SiteShell } from "@/components/layout/SiteShell";
import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";
import { LocaleLayoutSync } from "@/components/motion/LocaleLayoutSync";
import { INITIAL_HASH_BOOTSTRAP_SCRIPT } from "@/lib/motion/initialHash";
import { getLocaleFromCookieValue, LOCALE_COOKIE_NAME } from "@/i18n/config";
import { LanguageProvider } from "@/i18n/LanguageProvider";

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

export default async function RootLayout({ children }: LayoutProps<"/">) {
  const cookieStore = await cookies();
  const initialLocale = getLocaleFromCookieValue(
    cookieStore.get(LOCALE_COOKIE_NAME)?.value,
  );

  return (
    <html
      lang={initialLocale}
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
        <LanguageProvider initialLocale={initialLocale}>
          <LocaleLayoutSync />
          <SmoothScrollProvider>
            <SiteShell>{children}</SiteShell>
          </SmoothScrollProvider>
        </LanguageProvider>
      </body>
    </html>
  );
}
