import { forwardDashboardRequest } from "@/lib/admin/adminBackend";

export const GET = (request: Request) => forwardDashboardRequest(request);
