import { fireEvent, screen } from "@testing-library/react";

import { AdminWorkspace } from "@/components/admin/shell/AdminWorkspace";
import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { render } from "@/tests/renderWithLanguage";

describe("Staff Workspace copy", () => {
  it("uses timeless product copy and updates it when the application language changes", () => {
    const { container } = render(<><LanguageToggle /><AdminWorkspace /></>);

    expect(screen.getByRole("heading", { name: "Staff workspace" })).toBeInTheDocument();
    expect(screen.getByText("Authorized operations available")).toBeInTheDocument();
    expect(screen.getByText("Your authorized X-Fly operational modules are available from the navigation based on your assigned responsibilities.")).toBeInTheDocument();
    expect(container).not.toHaveTextContent("Branch 19");

    fireEvent.click(screen.getByRole("button", { name: "Current language: English. Switch to Thai." }));
    expect(screen.getByRole("heading", { name: "พื้นที่ทำงานสำหรับพนักงาน" })).toBeInTheDocument();
    expect(screen.getByText("พร้อมใช้งานระบบปฏิบัติการที่ได้รับอนุญาต")).toBeInTheDocument();
    expect(screen.getByText("โมดูลปฏิบัติการ X-Fly ที่คุณได้รับอนุญาตพร้อมใช้งานจากเมนูนำทางตามหน้าที่ที่ได้รับมอบหมาย")).toBeInTheDocument();
    expect(container).not.toHaveTextContent("Branch 19");
  });
});
