import { forwardAdminTicketRequest } from "@/lib/admin/adminBackend";
export const POST=(request:Request)=>forwardAdminTicketRequest(request,"/admin/tickets/search");
