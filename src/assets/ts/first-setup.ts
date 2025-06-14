import $ from "jquery";

export async function hasCompletedFirstSetup(): Promise<boolean>
{
    let response = await $.get("/api/ic-config/");
    if (response && response.first_setup_completed !== undefined)
    {
        return response.first_setup_completed;
    }
    return false;
}

export async function completeFirstSetup()
{
    await $.post("/api/ic-config/complete-first-run-setup");
}