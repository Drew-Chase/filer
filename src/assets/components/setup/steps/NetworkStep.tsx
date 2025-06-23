import {m} from "framer-motion";
import {useSetup} from "../../../providers/SetupProvider.tsx";
import {Button, Chip, Input, Link, NumberInput, Spinner, Switch, Textarea} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";

interface NetworkSettings
{
    port: number;
    upnp_enabled: boolean;
    authorized_hosts: string[];
    cors_enabled: boolean;
}

interface CreateNetworkConfigRequest
{
    port: number;
    root_path: string;
    indexing_enabled: boolean;
    file_watcher_enabled: boolean;
    filter_mode_whitelist: boolean;
    filter: string[];
    included_extensions: string[];
    exclude_hidden_files: boolean;
    // Network configuration fields
    upnp_enabled: boolean;
    authorized_hosts: string[];
    cors_enabled: boolean;
}

export default function NetworkStep()
{
    const {completeStep, gotoStep, currentIndex} = useSetup();

    const [networkSettings, setNetworkSettings] = useState<NetworkSettings>({
        port: 7667,
        upnp_enabled: false,
        authorized_hosts: ["127.0.0.1", "localhost", "::1"],
        cors_enabled: true
    });

    const [hostInput, setHostInput] = useState("");
    const [isSaving, setIsSaving] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [apiError, setApiError] = useState<string | null>(null);
    const [apiSuccess, setApiSuccess] = useState<string | null>(null);

    // Load current configuration on component mount
    useEffect(() =>
    {
        const loadCurrentConfig = async () =>
        {
            setIsLoading(true);
            setApiError(null);

            try
            {
                const response = await fetch("/api/config/?reload");

                if (!response.ok)
                {
                    throw new Error(`Failed to load configuration: ${response.status}`);
                }

                const config = await response.json();

                // Update state with loaded configuration
                setNetworkSettings({
                    port: config.port ?? 7667,
                    upnp_enabled: config.upnp_enabled ?? false,
                    authorized_hosts: config.authorized_hosts ?? ["127.0.0.1", "localhost", "::1"],
                    cors_enabled: config.cors_enabled ?? true
                });
            } catch (error)
            {
                console.error("Failed to load configuration:", error);
                setApiError(error instanceof Error ? error.message : "Failed to load current configuration");
                // Keep default values if loading fails
            } finally
            {
                setIsLoading(false);
            }
        };

        loadCurrentConfig().then();
    }, []);

    const handleAddHost = () =>
    {
        if (hostInput.trim() && !networkSettings.authorized_hosts.includes(hostInput.trim()))
        {
            setNetworkSettings(prev => ({
                ...prev,
                authorized_hosts: [...prev.authorized_hosts, hostInput.trim()]
            }));
            setHostInput("");
        }
    };

    const handleRemoveHost = (host: string) =>
    {
        setNetworkSettings(prev => ({
            ...prev,
            authorized_hosts: prev.authorized_hosts.filter(h => h !== host)
        }));
    };

    const saveConfiguration = async (configData: CreateNetworkConfigRequest): Promise<void> =>
    {
        const response = await fetch("/api/config/", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(configData)
        });

        if (!response.ok)
        {
            const error = await response.text();
            throw new Error(error || "Failed to save configuration");
        }
    };

    const handleSaveAndContinue = async () =>
    {
        setIsSaving(true);
        setApiError(null);
        setApiSuccess(null);

        try
        {
            // First, load the current full configuration
            const response = await fetch("/api/config/?reload");
            if (!response.ok)
            {
                throw new Error("Failed to load current configuration");
            }
            const currentConfig = await response.json();

            // Merge network settings with the existing configuration
            await saveConfiguration({
                ...currentConfig,
                port: networkSettings.port,
                upnp_enabled: networkSettings.upnp_enabled,
                authorized_hosts: networkSettings.authorized_hosts,
                cors_enabled: networkSettings.cors_enabled
            });

            setApiSuccess("Network configuration saved successfully!");

            // Wait a moment to show the success message, then continue
            completeStep(currentIndex);
            gotoStep(currentIndex + 1);
        } catch (error)
        {
            setApiError(error instanceof Error ? error.message : "Failed to save network configuration");
        } finally
        {
            setIsSaving(false);
        }
    };

    // Show loading state while fetching configuration
    if (isLoading)
    {
        return (
            <m.div
                key="network-loading"
                className={"h-full w-full bg-white/5 rounded-xl shadow-xl border border-white/20 overflow-hidden flex items-center justify-center"}
                initial={{opacity: 0}}
                animate={{opacity: 1}}
                transition={{duration: 0.25}}
            >
                <div className={"flex flex-row items-center gap-4"}>
                    <Spinner size={"md"}/>
                    <span className={"text-lg text-default-500"}>Loading current configuration...</span>
                </div>
            </m.div>
        );
    }

    return (
        <m.div
            key="networking"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl border border-white/20 overflow-hidden relative"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            {/* Fixed Buttons */}
            <Button
                variant={"light"}
                size={"lg"}
                radius={"full"}
                className={"w-12 h-12 absolute top-4 right-4 aspect-square text-large p-0 min-w-0 min-h-0 z-50"}
                as={Link}
                href={"https://github.com/Drew-Chase/filer/wiki/Setup#configure-your-network"}
                target={"_blank"}
            >
                <Icon icon={"mingcute:question-fill"}/>
            </Button>

            <Button
                color={"secondary"}
                size={"lg"}
                variant={"solid"}
                radius={"full"}
                className={"w-12 h-12 absolute bottom-4 right-4 aspect-square p-0 min-w-0 min-h-0 z-50"}
                onPress={handleSaveAndContinue}
                isLoading={isSaving}
                isDisabled={isSaving}
            >
                {isSaving ? null : <Icon icon={"maki:arrow"}/>}
            </Button>

            {/* Scrollable Content */}
            <div
                className={"h-full w-full overflow-y-scroll p-6"}
            >
                {/* Animated Title */}
                <h1 className={"text-7xl font-bold text-center origin-center"}>
                    Network Configuration
                </h1>

                {/* Error/Success Messages */}
                {apiError && (
                    <div className={"mb-4 p-3 bg-red-500/20 border border-red-500/40 rounded-lg text-red-300 text-sm mt-6 flex flex-row items-center"}>
                        <Icon icon={"mdi:alert-circle"} className={"inline mr-2"}/>
                        {apiError}
                    </div>
                )}

                {apiSuccess && (
                    <div className={"mb-4 p-3 bg-green-500/20 border border-green-500/40 rounded-lg text-green-300 text-sm mt-6 flex flex-row items-center"}>
                        <Icon icon={"mdi:check-circle"} className={"inline mr-2"}/>
                        {apiSuccess}
                    </div>
                )}

                {/* Content with top padding to account for shrunk title */}
                <div className={"w-full flex flex-col gap-6 items-center justify-start pb-20 pt-8"}>
                    <div className={"w-full max-w-2xl space-y-6"}>
                        {/* Server Port */}
                        <m.div
                            className={"space-y-2"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.2}}
                        >
                            <NumberInput
                                label="Server Port"
                                placeholder="7667"
                                value={networkSettings.port}
                                onValueChange={(value) => setNetworkSettings(prev => ({...prev, port: value}))}
                                startContent={<Icon icon={"mdi:port"} className={"text-default-400"}/>}
                                description="Port number for the server to listen on"
                                variant="bordered"
                                min={1}
                                max={65535}
                                formatOptions={{useGrouping: false}}
                            />
                        </m.div>

                        {/* Network Switches */}
                        <m.div
                            className={"space-y-4 p-4 bg-white/5 rounded-lg border border-white/10"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.4}}
                        >
                            <h3 className={"text-xl font-semibold mb-4"}>Network Features</h3>

                            <div className={"flex items-center justify-between"}>
                                <div className={"flex flex-col cursor-pointer"} onClick={() => setNetworkSettings(prev => ({...prev, upnp_enabled: !prev.upnp_enabled}))}>
                                    <span className={"font-medium"}>Enable UPnP</span>
                                    <span className={"text-sm text-default-500"}>Automatically configure port forwarding</span>
                                </div>
                                <Switch
                                    isSelected={networkSettings.upnp_enabled}
                                    onValueChange={(value) => setNetworkSettings(prev => ({...prev, upnp_enabled: value}))}
                                />
                            </div>

                            <div className={"flex items-center justify-between"}>
                                <div className={"flex flex-col cursor-pointer"} onClick={() => setNetworkSettings(prev => ({...prev, cors_enabled: !prev.cors_enabled}))}>
                                    <span className={"font-medium"}>Enable CORS</span>
                                    <span className={"text-sm text-default-500"}>Allow cross-origin requests</span>
                                </div>
                                <Switch
                                    isSelected={networkSettings.cors_enabled}
                                    onValueChange={(value) => setNetworkSettings(prev => ({...prev, cors_enabled: value}))}
                                />
                            </div>
                        </m.div>

                        {/* Authorized Hosts */}
                        <m.div
                            className={"space-y-4"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.5}}
                        >
                            <div className={"space-y-2"}>
                                <label className={"text-sm font-medium"}>Authorized Hosts</label>
                                <div className={"flex gap-2"}>
                                    <Input
                                        placeholder="Enter IP address or hostname"
                                        value={hostInput}
                                        onChange={(e) => setHostInput(e.target.value)}
                                        onKeyDown={(e) => e.key === "Enter" && handleAddHost()}
                                        startContent={<Icon icon={"mdi:web"} className={"text-default-400"}/>}
                                        variant="bordered"
                                        className={"flex-1"}
                                    />
                                    <Button
                                        color="secondary"
                                        variant="solid"
                                        onPress={handleAddHost}
                                        className={"px-4"}
                                    >
                                        <Icon icon={"mdi:plus"}/>
                                    </Button>
                                </div>
                                <p className={"text-sm text-default-500"}>
                                    Specify which hosts are allowed to connect to your server
                                </p>
                            </div>

                            {/* Host Chips */}
                            <div className={"flex flex-wrap gap-2 p-3 bg-white/5 rounded-lg border border-white/10 min-h-[60px]"}>
                                {networkSettings.authorized_hosts.length === 0 ? (
                                    <span className={"text-default-500 text-sm"}>No hosts configured</span>
                                ) : (
                                    networkSettings.authorized_hosts.map((host, index) => (
                                        <Chip
                                            key={index}
                                            onClose={() => handleRemoveHost(host)}
                                            variant="flat"
                                            color="secondary"
                                            className={"text-sm"}
                                        >
                                            {host}
                                        </Chip>
                                    ))
                                )}
                            </div>
                        </m.div>

                        {/* Configuration Preview */}
                        <m.div
                            className={"space-y-2"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.6}}
                        >
                            <label className={"text-sm font-medium"}>Configuration Preview</label>
                            <Textarea
                                value={JSON.stringify(networkSettings, null, 2)}
                                readOnly
                                variant="bordered"
                                className={"font-mono text-sm"}
                                minRows={8}
                                maxRows={12}
                            />
                        </m.div>
                    </div>
                </div>
            </div>
        </m.div>
    );
}