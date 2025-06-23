import {m} from "framer-motion";
import {useSetup} from "../../../providers/SetupProvider.tsx";
import {Button, Chip, Input, Link, Radio, RadioGroup, Select, SelectItem, Spinner, Switch, Textarea} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";
import SelectDirectoryEntryModal from "../../modals/SelectDirectoryEntryModal.tsx";

interface StorageSettings
{
    root_path: string;
    indexing_enabled: boolean;
    file_watcher_enabled: boolean;
    filter_mode_whitelist: boolean;
    filter: string[];
    included_extensions: string[];
    exclude_hidden_files: boolean;
}

interface CreateStorageConfigRequest
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

type FilterPattern = {
    label: string;
    value: string;
}

export default function StorageStep()
{
    const {completeStep, gotoStep, currentIndex} = useSetup();

    const [storageSettings, setStorageSettings] = useState<StorageSettings>({
        root_path: "/",
        indexing_enabled: true,
        file_watcher_enabled: true,
        filter_mode_whitelist: false,
        filter: [
            "/dev/**/*",
            "/proc/**/*",
            "/sys/**/*",
            "/run/**/*",
            "/mnt/**/*",
            "/media/**/*",
            "/var/log/**/*",
            "/var/cache/**/*",
            "C:/Windows/**/*",
            "C:/Program Files/Windows Defender/**/*",
            "C:/ProgramData/Microsoft/**/*",
            "**/*.log",
            "**/*.tmp",
            "**/*.bak",
            "**/Temp/**",
            "**/tmp/**"
        ],
        included_extensions: [".txt", ".pdf", ".doc", ".docx", ".jpg", ".png", ".mp4", ".mp3"],
        exclude_hidden_files: true
    });

    const [filterInput, setFilterInput] = useState("");
    const [extensionInput, setExtensionInput] = useState("");
    const [isSaving, setIsSaving] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [apiError, setApiError] = useState<string | null>(null);
    const [apiSuccess, setApiSuccess] = useState<string | null>(null);
    const [isSelectingStorageRoot, setIsSelectingStorageRoot] = useState(false);

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
                setStorageSettings({
                    root_path: config.root_path ?? "/",
                    indexing_enabled: config.indexing_enabled ?? true,
                    file_watcher_enabled: config.file_watcher_enabled ?? true,
                    filter_mode_whitelist: config.filter_mode_whitelist ?? false,
                    filter: config.filter ?? [],
                    included_extensions: config.included_extensions ?? [],
                    exclude_hidden_files: config.exclude_hidden_files ?? true
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

    const handleAddFilter = () =>
    {
        if (filterInput.trim() && !storageSettings.filter.includes(filterInput.trim()))
        {
            setStorageSettings(prev => ({
                ...prev,
                filter: [...prev.filter, filterInput.trim()]
            }));
            setFilterInput("");
        }
    };

    const handleRemoveFilter = (filter: string) =>
    {
        setStorageSettings(prev => ({
            ...prev,
            filter: prev.filter.filter(f => f !== filter)
        }));
    };

    const handleAddExtension = () =>
    {
        const ext = extensionInput.trim();
        if (ext && !storageSettings.included_extensions.includes(ext))
        {
            const formattedExt = ext.startsWith(".") ? ext : `.${ext}`;
            setStorageSettings(prev => ({
                ...prev,
                included_extensions: [...prev.included_extensions, formattedExt]
            }));
            setExtensionInput("");
        }
    };

    const handleRemoveExtension = (extension: string) =>
    {
        setStorageSettings(prev => ({
            ...prev,
            included_extensions: prev.included_extensions.filter(e => e !== extension)
        }));
    };


    const saveConfiguration = async (configData: CreateStorageConfigRequest): Promise<void> =>
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

            // Merge storage settings with the existing configuration
            await saveConfiguration({
                ...currentConfig,
                root_path: storageSettings.root_path,
                indexing_enabled: storageSettings.indexing_enabled,
                file_watcher_enabled: storageSettings.file_watcher_enabled,
                filter_mode_whitelist: storageSettings.filter_mode_whitelist,
                filter: storageSettings.filter,
                included_extensions: storageSettings.included_extensions,
                exclude_hidden_files: storageSettings.exclude_hidden_files
            });

            setApiSuccess("Storage configuration saved successfully!");

            // Wait a moment to show the success message, then continue
            completeStep(currentIndex);
            gotoStep(currentIndex + 1);
        } catch (error)
        {
            setApiError(error instanceof Error ? error.message : "Failed to save storage configuration");
        } finally
        {
            setIsSaving(false);
        }
    };

    const commonFilterPatterns: FilterPattern[] = [
        {label: "System Files", value: "/sys/**/*"},
        {label: "Log Files", value: "**/*.log"},
        {label: "Temporary Files", value: "**/*.tmp"},
        {label: "Backup Files", value: "**/*.bak"},
        {label: "Cache Directories", value: "**/cache/**"},
        {label: "Node Modules", value: "**/node_modules/**"},
        {label: "Git Directories", value: "**/.git/**"}
    ];

    // Show loading state while fetching configuration
    if (isLoading)
    {
        return (
            <m.div
                key="storage-loading"
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
            key="storage"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl border border-white/20 overflow-hidden relative"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            <SelectDirectoryEntryModal
                isOpen={isSelectingStorageRoot}
                onClose={
                    (newRootPath) =>
                    {
                        setIsSelectingStorageRoot(false);
                        if (newRootPath)
                        {
                            setStorageSettings(prev => ({...prev, root_path: newRootPath}));
                        }
                    }
                }
                label={"Storage Root"}
            />
            {/* Fixed Buttons */}
            <Button
                variant={"light"}
                size={"lg"}
                radius={"full"}
                className={"w-12 h-12 absolute top-4 right-4 aspect-square text-large p-0 min-w-0 min-h-0 z-50"}
                as={Link}
                href={"https://github.com/Drew-Chase/filer/wiki/Setup#configure-your-storage"}
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
                    Storage Configuration
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

                {/* Content */}
                <div className={"w-full flex flex-col gap-6 items-center justify-start pb-20 pt-8"}>
                    <div className={"w-full max-w-2xl space-y-6"}>
                        {/* Root Path */}
                        <m.div
                            className={"space-y-2"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.2}}
                        >
                            <div className={"flex gap-2"}>
                                <Input
                                    label="Root Path"
                                    placeholder="/"
                                    value={storageSettings.root_path}
                                    onChange={(e) => setStorageSettings(prev => ({...prev, root_path: e.target.value}))}
                                    startContent={<Icon icon={"mdi:folder-outline"} className={"text-default-400"}/>}
                                    description="The root directory to serve files from"
                                    variant="bordered"
                                    className={"flex-1"}
                                    endContent={
                                        <Button
                                            color="secondary"
                                            variant="light"
                                            onPress={() => setIsSelectingStorageRoot(true)}
                                        >
                                            <Icon icon={"mdi:folder-search"}/>
                                        </Button>
                                    }
                                />
                            </div>
                        </m.div>

                        {/* Core Features */}
                        <m.div
                            className={"space-y-4 p-4 bg-white/5 rounded-lg border border-white/10"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.3}}
                        >
                            <h3 className={"text-xl font-semibold mb-4"}>Core Features</h3>

                            <div className={"flex items-center justify-between"}>
                                <div className={"flex flex-col cursor-pointer"} onClick={() => setStorageSettings(prev => ({...prev, indexing_enabled: !prev.indexing_enabled}))}>
                                    <span className={"font-medium"}>Enable Indexing</span>
                                    <span className={"text-sm text-default-500"}>Build search index for faster file discovery</span>
                                </div>
                                <Switch
                                    isSelected={storageSettings.indexing_enabled}
                                    onValueChange={(value) => setStorageSettings(prev => ({...prev, indexing_enabled: value}))}
                                />
                            </div>

                            <div className={"flex items-center justify-between"}>
                                <div className={"flex flex-col cursor-pointer"} onClick={() => setStorageSettings(prev => ({...prev, file_watcher_enabled: !prev.file_watcher_enabled}))}>
                                    <span className={"font-medium"}>Enable File Watcher</span>
                                    <span className={"text-sm text-default-500"}>Monitor file system changes in real-time</span>
                                </div>
                                <Switch
                                    isSelected={storageSettings.file_watcher_enabled}
                                    onValueChange={(value) => setStorageSettings(prev => ({...prev, file_watcher_enabled: value}))}
                                />
                            </div>

                            <div className={"flex items-center justify-between"}>
                                <div className={"flex flex-col cursor-pointer"} onClick={() => setStorageSettings(prev => ({...prev, exclude_hidden_files: !prev.exclude_hidden_files}))}>
                                    <span className={"font-medium"}>Exclude Hidden Files</span>
                                    <span className={"text-sm text-default-500"}>Skip files and directories starting with .</span>
                                </div>
                                <Switch
                                    isSelected={storageSettings.exclude_hidden_files}
                                    onValueChange={(value) => setStorageSettings(prev => ({...prev, exclude_hidden_files: value}))}
                                />
                            </div>
                        </m.div>

                        {/* Filter Mode */}
                        <m.div
                            className={"space-y-4"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.5}}
                        >
                            <div className={"space-y-2"}>
                                <label className={"text-sm font-medium"}>Filter Mode</label>
                                <RadioGroup
                                    value={storageSettings.filter_mode_whitelist ? "whitelist" : "blacklist"}
                                    onValueChange={(value) => setStorageSettings(prev => ({...prev, filter_mode_whitelist: value === "whitelist"}))}
                                    orientation="horizontal"
                                    className={"gap-6"}
                                >
                                    <Radio value="blacklist" className={"text-sm"}>
                                        <div className={"flex flex-col"}>
                                            <span className={"font-medium"}>Blacklist</span>
                                            <span className={"text-xs text-default-500"}>Exclude specified patterns</span>
                                        </div>
                                    </Radio>
                                    <Radio value="whitelist" className={"text-sm"}>
                                        <div className={"flex flex-col"}>
                                            <span className={"font-medium"}>Whitelist</span>
                                            <span className={"text-xs text-default-500"}>Include only specified patterns</span>
                                        </div>
                                    </Radio>
                                </RadioGroup>
                            </div>
                        </m.div>

                        {/* Filter List */}
                        <m.div
                            className={"space-y-4"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.6}}
                        >
                            <div className={"space-y-2"}>
                                <label className={"text-sm font-medium"}>
                                    Filter Patterns {storageSettings.filter_mode_whitelist ? "(Whitelist)" : "(Blacklist)"}
                                </label>
                                <div className={"flex gap-2"}>
                                    <Input
                                        placeholder="Enter glob pattern (e.g., **/*.tmp)"
                                        value={filterInput}
                                        onChange={(e) => setFilterInput(e.target.value)}
                                        onKeyDown={(e) => e.key === "Enter" && handleAddFilter()}
                                        startContent={<Icon icon={"mdi:filter-outline"} className={"text-default-400"}/>}
                                        variant="bordered"
                                        className={"flex-1"}
                                    />
                                    <Select
                                        placeholder="Quick add"
                                        className={"w-40"}
                                        variant="bordered"
                                        onSelectionChange={(keys) =>
                                        {
                                            const selectedKey = Array.from(keys)[0] as string;
                                            if (selectedKey && !storageSettings.filter.includes(selectedKey))
                                            {
                                                setStorageSettings(prev => ({
                                                    ...prev,
                                                    filter: [...prev.filter, selectedKey]
                                                }));
                                            }
                                        }}
                                    >
                                        {commonFilterPatterns.map((pattern) => (
                                            <SelectItem key={pattern.value} textValue={pattern.value}>
                                                {pattern.label}
                                            </SelectItem>
                                        ))}
                                    </Select>
                                    <Button
                                        color="secondary"
                                        variant="solid"
                                        onPress={handleAddFilter}
                                        className={"px-4"}
                                    >
                                        <Icon icon={"mdi:plus"}/>
                                    </Button>
                                </div>
                                <p className={"text-sm text-default-500"}>
                                    Use glob patterns to {storageSettings.filter_mode_whitelist ? "include" : "exclude"} files and directories
                                </p>
                            </div>

                            {/* Filter Chips */}
                            <div className={"flex flex-wrap gap-2 p-3 bg-white/5 rounded-lg border border-white/10 min-h-[60px] max-h-40 overflow-y-auto"}>
                                {storageSettings.filter.length === 0 ? (
                                    <span className={"text-default-500 text-sm"}>No filters configured</span>
                                ) : (
                                    storageSettings.filter.map((filter, index) => (
                                        <Chip
                                            key={index}
                                            onClose={() => handleRemoveFilter(filter)}
                                            variant="flat"
                                            color={storageSettings.filter_mode_whitelist ? "success" : "danger"}
                                            className={"text-sm"}
                                        >
                                            {filter}
                                        </Chip>
                                    ))
                                )}
                            </div>
                        </m.div>

                        {/* File Extensions */}
                        <m.div
                            className={"space-y-4"}
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.7}}
                        >
                            <div className={"space-y-2"}>
                                <label className={"text-sm font-medium"}>Indexed File Extensions</label>
                                <div className={"flex gap-2"}>
                                    <Input
                                        placeholder="Enter file extension (e.g., .pdf)"
                                        value={extensionInput}
                                        onChange={(e) => setExtensionInput(e.target.value)}
                                        onKeyDown={(e) => e.key === "Enter" && handleAddExtension()}
                                        startContent={<Icon icon={"mdi:file-document-outline"} className={"text-default-400"}/>}
                                        variant="bordered"
                                        className={"flex-1"}
                                    />
                                    <Button
                                        color="secondary"
                                        variant="solid"
                                        onPress={handleAddExtension}
                                        className={"px-4"}
                                    >
                                        <Icon icon={"mdi:plus"}/>
                                    </Button>
                                </div>
                                <p className={"text-sm text-default-500"}>
                                    Specify which file types to include in indexing
                                </p>
                            </div>

                            {/* Extension Chips */}
                            <div className={"flex flex-wrap gap-2 p-3 bg-white/5 rounded-lg border border-white/10 min-h-[60px]"}>
                                {storageSettings.included_extensions.length === 0 ? (
                                    <span className={"text-default-500 text-sm"}>No extensions specified (all files)</span>
                                ) : (
                                    storageSettings.included_extensions.map((extension, index) => (
                                        <Chip
                                            key={index}
                                            onClose={() => handleRemoveExtension(extension)}
                                            variant="flat"
                                            color="primary"
                                            className={"text-sm"}
                                        >
                                            {extension}
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
                            transition={{delay: 0.8}}
                        >
                            <label className={"text-sm font-medium"}>Configuration Preview</label>
                            <Textarea
                                value={JSON.stringify(storageSettings, null, 2)}
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