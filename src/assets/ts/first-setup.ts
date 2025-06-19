import $ from "jquery";

export async function hasCompletedFirstSetup(): Promise<boolean>
{
    let response = await $.get("/api/ic-config/");
    if (response && response.has_done_first_run_setup !== undefined)
    {
        return response.has_done_first_run_setup;
    }
    return false;
}

export async function completeFirstSetup()
{
    await $.post("/api/ic-config/complete-first-run-setup");
}