export type Configuration = {
    "port": number,
    "root_path": string,
    "indexing_enabled": boolean,
    "file_watcher_enabled": boolean,
    "filter_mode_whitelist": boolean,
    "filter": string[],
    "included_extensions": string[],
    "exclude_hidden_files": boolean,
    "http_root_path": string,
    "upnp_enabled": boolean,
    "authorized_hosts": string[],
    "cors_enabled": boolean
}

export async function getConfiguration(reload: boolean = false): Promise<Configuration>
{
    let response = await fetch(`/api/config/${reload ? "?reload" : ""}`);
    if (!response.ok)
    {
        throw new Error("Failed to fetch configuration: " + response.statusText);
    }
    return await response.json();
}

export async function updateConfiguration(config: Configuration): Promise<void>
{
    let response = await fetch("/api/config", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(config)
    });
    if (!response.ok)
    {
        throw new Error("Failed to update configuration: " + response.statusText);
    }
}