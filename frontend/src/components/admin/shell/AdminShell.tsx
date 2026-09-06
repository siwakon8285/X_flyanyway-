"use client";

import { Menu, ShieldCheck } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { useState } from "react";

import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { AdminLogoutButton } from "@/components/admin/auth/AdminLogoutButton";
import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { Badge } from "@/components/ui/Badge";
import {
  Dialog, DialogClose, DialogContent, DialogDescription, DialogTitle, DialogTrigger,
} from "@/components/ui/Dialog";
import { useLanguage } from "@/i18n/LanguageProvider";
import { getVisibleAdminNavigation } from "@/lib/admin/adminAuthorization";
import type { StaffPrincipal, StaffRole } from "@/lib/admin/adminTypes";

type AdminShellProps = { children: ReactNode; principal: StaffPrincipal };

const roleTranslationKeys: Record<StaffRole,
  | "admin.roles.executive" | "admin.roles.flightManager" | "admin.roles.bookingOperations"
  | "admin.roles.ticketPassengerOperations" | "admin.roles.baggageStaff"
  | "admin.roles.apiAdmin" | "admin.roles.systemAdmin"
> = {
  EXECUTIVE: "admin.roles.executive",
  FLIGHT_MANAGER: "admin.roles.flightManager",
  BOOKING_OPERATIONS: "admin.roles.bookingOperations",
  TICKET_PASSENGER_OPERATIONS: "admin.roles.ticketPassengerOperations",
  BAGGAGE_STAFF: "admin.roles.baggageStaff",
  API_ADMIN: "admin.roles.apiAdmin",
  SYSTEM_ADMIN: "admin.roles.systemAdmin",
};

const AdminNavigation = ({ principal, onNavigate }: { principal: StaffPrincipal; onNavigate?: () => void }) => {
  const { t } = useLanguage();
  return <nav aria-label={t("admin.navigation.label")}>
    <ul className="space-y-2">
      {getVisibleAdminNavigation(principal).map((item) => <li key={item.id}>
        <Link className="flex min-h-11 items-center rounded-control border border-brand/20 bg-brand/10 px-4 text-sm font-semibold text-brand outline-none transition-colors hover:bg-brand/15 focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none" href={item.href} onClick={onNavigate}>
          {t(item.labelKey)}
        </Link>
      </li>)}
    </ul>
  </nav>;
};

const Identity = ({ principal }: { principal: StaffPrincipal }) => {
  const { t } = useLanguage();
  return <div className="space-y-3">
    <p className="break-all text-sm font-medium text-foreground">{principal.email}</p>
    <div className="flex flex-wrap gap-2">
      {principal.roles.map((role) => <Badge key={role} variant="brand">{t(roleTranslationKeys[role])}</Badge>)}
    </div>
  </div>;
};

const AdminShell = ({ children, principal }: AdminShellProps) => {
  const { t } = useLanguage();
  const [open, setOpen] = useState(false);
  return <div className="min-h-dvh bg-[#f6f3e9] text-[#171717]">
    <a className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-[70] focus:rounded-control focus:bg-brand focus:px-4 focus:py-3 focus:text-brand-foreground" href="#admin-content">{t("admin.shell.skip")}</a>
    <aside className="fixed inset-y-0 left-0 hidden w-72 border-r border-white/10 bg-[#101010] p-6 text-foreground lg:flex lg:flex-col">
      <BrandWordmark />
      <div className="mt-4 flex items-center gap-2 text-caption text-brand"><ShieldCheck aria-hidden="true" className="size-4" />{t("admin.shell.internal")}</div>
      <div className="mt-12"><AdminNavigation principal={principal} /></div>
      <div className="mt-auto border-t border-white/10 pt-6"><Identity principal={principal} /><AdminLogoutButton /></div>
    </aside>
    <div className="lg:pl-72">
      <header className="sticky top-0 z-30 flex min-h-20 items-center justify-between gap-4 border-b border-black/10 bg-[#f6f3e9]/95 px-4 backdrop-blur md:px-8">
        <div><p className="text-caption text-black/50">{t("admin.shell.terminal")}</p><p className="font-semibold">{t("admin.navigation.workspace")}</p></div>
        <div className="flex items-center gap-2"><LanguageToggle />
          <Dialog onOpenChange={setOpen} open={open}>
            <DialogTrigger asChild><button aria-label={t("admin.shell.openNavigation")} className="inline-flex size-11 items-center justify-center rounded-control border border-black/15 outline-none focus-visible:ring-2 focus-visible:ring-focus lg:hidden" type="button"><Menu aria-hidden="true" className="size-5" /></button></DialogTrigger>
            <DialogContent className="left-auto right-0 top-0 h-dvh w-[min(24rem,100%)] max-w-none translate-x-0 translate-y-0 rounded-none border-y-0 border-r-0 bg-[#101010] text-foreground" showCloseButton>
              <DialogTitle>{t("admin.shell.navigationTitle")}</DialogTitle>
              <DialogDescription>{t("admin.shell.navigationDescription")}</DialogDescription>
              <AdminNavigation principal={principal} onNavigate={() => setOpen(false)} />
              <div className="mt-auto"><Identity principal={principal} /><AdminLogoutButton /></div>
              <DialogClose className="sr-only">{t("admin.shell.closeNavigation")}</DialogClose>
            </DialogContent>
          </Dialog>
        </div>
      </header>
      <main className="mx-auto max-w-[100rem] p-4 md:p-8 lg:p-12" id="admin-content">{children}</main>
    </div>
  </div>;
};

export { AdminShell, roleTranslationKeys };
export type { AdminShellProps };
