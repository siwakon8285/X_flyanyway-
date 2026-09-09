import { forwardAdminTicketRequest } from "@/lib/admin/adminBackend";
export const GET=(request:Request)=>forwardAdminTicketRequest(request,"/admin/tickets");
