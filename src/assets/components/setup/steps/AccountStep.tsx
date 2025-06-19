import {m} from "framer-motion";
import {useSetup} from "../../../providers/SetupProvider.tsx";
import {Button, Card, CardBody, CardHeader, Chip, Divider, Input, Link, Switch} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useState} from "react";

interface User
{
    id?: number;
    username: string;
    password: string;
    permissions: string[];
}

interface CreateUserRequest
{
    username: string;
    password: string;
    permissions: string[];
}

interface AccountSettings
{
    createDefaultAdmin: boolean;
    adminUsername: string;
    adminPassword: string;
    users: User[];
}

export default function AccountStep()
{
    const {completeStep, gotoStep, currentIndex} = useSetup();

    const [accountSettings, setAccountSettings] = useState<AccountSettings>({
        createDefaultAdmin: true,
        adminUsername: "admin",
        adminPassword: "",
        users: []
    });

    const [newUser, setNewUser] = useState<User>({
        username: "",
        password: "",
        permissions: []
    });

    const [isCreatingUser, setIsCreatingUser] = useState(false);
    const [apiError, setApiError] = useState<string | null>(null);
    const [apiSuccess, setApiSuccess] = useState<string | null>(null);

    const availablePermissions = [
        {key: "Read", label: "Read", description: "Permission to read/view files"},
        {key: "Write", label: "Write", description: "Permission to write/modify files"},
        {key: "Delete", label: "Delete", description: "Permission to delete files"},
        {key: "Create", label: "Create", description: "Permission to create new files"},
        {key: "Upload", label: "Upload", description: "Permission to upload files"},
        {key: "Download", label: "Download", description: "Permission to download files"}
    ];

    const handlePermissionToggle = (permission: string, isAdmin = false) =>
    {
        if (isAdmin)
        {
            // For admin permissions, directly update admin permissions
            return;
        }

        setNewUser(prev => ({
            ...prev,
            permissions: prev.permissions.includes(permission)
                ? prev.permissions.filter(p => p !== permission)
                : [...prev.permissions, permission]
        }));
    };

    const createUser = async (userData: CreateUserRequest): Promise<void> =>
    {
        const response = await fetch("/api/auth/users", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(userData)
        });

        if (!response.ok)
        {
            const error = await response.json();
            throw new Error(error.error || "Failed to create user");
        }

        return response.json();
    };

    const handleAddUser = async () =>
    {
        if (!newUser.username.trim() || !newUser.password.trim())
        {
            setApiError("Username and password are required");
            return;
        }

        setIsCreatingUser(true);
        setApiError(null);
        setApiSuccess(null);

        try
        {
            await createUser({
                username: newUser.username,
                password: newUser.password,
                permissions: newUser.permissions
            });

            setAccountSettings(prev => ({
                ...prev,
                users: [...prev.users, {...newUser}]
            }));

            setNewUser({
                username: "",
                password: "",
                permissions: []
            });

            setApiSuccess(`User "${newUser.username}" created successfully`);
        } catch (error)
        {
            setApiError(error instanceof Error ? error.message : "Failed to create user");
        } finally
        {
            setIsCreatingUser(false);
        }
    };

    const handleRemoveUser = (username: string) =>
    {
        setAccountSettings(prev => ({
            ...prev,
            users: prev.users.filter(u => u.username !== username)
        }));
    };

    const handleCreateDefaultAdmin = async () =>
    {
        if (!accountSettings.adminPassword.trim())
        {
            setApiError("Admin password is required");
            return;
        }

        try
        {
            await createUser({
                username: accountSettings.adminUsername,
                password: accountSettings.adminPassword,
                permissions: ["Read", "Write", "Delete", "Create", "Upload", "Download"]
            });

            setApiSuccess(`Admin user "${accountSettings.adminUsername}" created successfully`);
        } catch (error)
        {
            setApiError(error instanceof Error ? error.message : "Failed to create admin user");
        }
    };

    const handleSaveAndContinue = async () =>
    {
        // Create default admin if enabled
        if (accountSettings.createDefaultAdmin && accountSettings.adminPassword.trim())
        {
            await handleCreateDefaultAdmin();
        }

        // Here you would typically save the settings to your backend or context
        console.log("Account settings:", accountSettings);
        completeStep(currentIndex);
        gotoStep(currentIndex + 1);
    };

    return (
        <m.div
            key="account"
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
                href={"https://github.com/Drew-Chase/filer/wiki/Setup#configure-your-accounts"}
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
            >
                <Icon icon={"maki:arrow"}/>
            </Button>

            {/* Scrollable Content */}
            <div className={"h-full w-full overflow-y-scroll p-6"}>
                {/* Title */}
                <h1 className={"text-7xl font-bold text-center mb-8"}>
                    Setup your Account
                </h1>

                {/* Error/Success Messages */}
                {apiError && (
                    <div className={"mb-4 p-3 bg-red-500/20 border border-red-500/40 rounded-lg text-red-300 text-sm flex flex-row items-center"}>
                        <Icon icon={"mdi:alert-circle"} className={"inline mr-2"}/>
                        {apiError}
                    </div>
                )}

                {apiSuccess && (
                    <div className={"mb-4 p-3 bg-green-500/20 border border-green-500/40 rounded-lg text-green-300 text-sm flex flex-row items-center"}>
                        <Icon icon={"mdi:check-circle"} className={"inline mr-2"}/>
                        {apiSuccess}
                    </div>
                )}

                {/* Content */}
                <div className={"w-full flex flex-col gap-6 items-center justify-start pb-20"}>
                    <div className={"w-full max-w-2xl space-y-6"}>

                        {/* Default Admin User */}
                        <m.div
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.1}}
                        >
                            <Card className={"bg-white/5 border border-white/10"}>
                                <CardHeader className={"pb-2"}>
                                    <div className={"flex items-center justify-between w-full"}>
                                        <div className={"flex items-center gap-3 cursor-pointer"} onClick={() => setAccountSettings(prev => ({...prev, createDefaultAdmin: !prev.createDefaultAdmin}))}>
                                            <Icon icon={"mdi:shield-account"} className={"text-2xl text-warning"}/>
                                            <div>
                                                <h3 className={"text-lg font-semibold"}>Default Admin User</h3>
                                                <p className={"text-sm text-default-500"}>Create a default administrator account</p>
                                            </div>
                                        </div>
                                        <Switch
                                            isSelected={accountSettings.createDefaultAdmin}
                                            onValueChange={(value) => setAccountSettings(prev => ({...prev, createDefaultAdmin: value}))}
                                        />
                                    </div>
                                </CardHeader>
                                {accountSettings.createDefaultAdmin && (
                                    <CardBody className={"pt-0"}>
                                        <div className={"space-y-4"}>
                                            <Input
                                                label="Admin Username"
                                                placeholder="admin"
                                                value={accountSettings.adminUsername}
                                                onChange={(e) => setAccountSettings(prev => ({...prev, adminUsername: e.target.value}))}
                                                startContent={<Icon icon={"mdi:account"} className={"text-default-400"}/>}
                                                variant="bordered"
                                            />
                                            <Input
                                                label="Admin Password"
                                                type="password"
                                                placeholder="Enter a secure password"
                                                value={accountSettings.adminPassword}
                                                onChange={(e) => setAccountSettings(prev => ({...prev, adminPassword: e.target.value}))}
                                                startContent={<Icon icon={"mdi:lock"} className={"text-default-400"}/>}
                                                variant="bordered"
                                                description="This user will have all permissions"
                                            />
                                            <div className={"text-sm text-default-500"}>
                                                <strong>Admin permissions include:</strong> Read, Write, Delete, Create, Upload, Download
                                            </div>
                                        </div>
                                    </CardBody>
                                )}
                            </Card>
                        </m.div>

                        <Divider className={"bg-white/20"}/>

                        {/* Add New User */}
                        <m.div
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.2}}
                        >
                            <Card className={"bg-white/5 border border-white/10"}>
                                <CardHeader>
                                    <div className={"flex items-center gap-3"}>
                                        <Icon icon={"mdi:account-plus"} className={"text-2xl text-secondary"}/>
                                        <div>
                                            <h3 className={"text-lg font-semibold"}>Add New User</h3>
                                            <p className={"text-sm text-default-500"}>Create additional user accounts</p>
                                        </div>
                                    </div>
                                </CardHeader>
                                <CardBody>
                                    <div className={"space-y-4"}>
                                        <div className={"grid grid-cols-1 md:grid-cols-2 gap-4"}>
                                            <Input
                                                label="Username"
                                                placeholder="Enter username"
                                                value={newUser.username}
                                                onChange={(e) => setNewUser(prev => ({...prev, username: e.target.value}))}
                                                startContent={<Icon icon={"mdi:account"} className={"text-default-400"}/>}
                                                variant="bordered"
                                            />
                                            <Input
                                                label="Password"
                                                type="password"
                                                placeholder="Enter password"
                                                value={newUser.password}
                                                onChange={(e) => setNewUser(prev => ({...prev, password: e.target.value}))}
                                                startContent={<Icon icon={"mdi:lock"} className={"text-default-400"}/>}
                                                variant="bordered"
                                            />
                                        </div>

                                        <div className={"space-y-3"}>
                                            <label className={"text-sm font-medium"}>Permissions</label>
                                            <div className={"grid grid-cols-2 md:grid-cols-3 gap-3"}>
                                                {availablePermissions.map(permission => (
                                                    <div
                                                        key={permission.key}
                                                        className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                                                            newUser.permissions.includes(permission.key)
                                                                ? "bg-secondary/20 border-secondary/40"
                                                                : "bg-white/5 border-white/10 hover:border-white/20"
                                                        }`}
                                                        onClick={() => handlePermissionToggle(permission.key)}
                                                    >
                                                        <div className={"flex items-center justify-between"}>
                                                            <div>
                                                                <div className={"font-medium text-sm"}>{permission.label}</div>
                                                                <div className={"text-xs text-default-500"}>{permission.description}</div>
                                                            </div>
                                                            <div className={`w-4 h-4 rounded border-2 flex items-center justify-center ${
                                                                newUser.permissions.includes(permission.key)
                                                                    ? "bg-secondary border-secondary"
                                                                    : "border-default-300"
                                                            }`}>
                                                                {newUser.permissions.includes(permission.key) && (
                                                                    <Icon icon={"mdi:check"} className={"text-xs text-black"}/>
                                                                )}
                                                            </div>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>

                                        <Button
                                            color="secondary"
                                            variant="solid"
                                            onPress={handleAddUser}
                                            isLoading={isCreatingUser}
                                            isDisabled={!newUser.username.trim() || !newUser.password.trim()}
                                            className={"w-full"}
                                            startContent={!isCreatingUser && <Icon icon={"mdi:account-plus"}/>}
                                        >
                                            {isCreatingUser ? "Creating User..." : "Add User"}
                                        </Button>
                                    </div>
                                </CardBody>
                            </Card>
                        </m.div>

                        {/* Existing Users */}
                        {accountSettings.users.length > 0 && (
                            <m.div
                                initial={{opacity: 0, y: 20}}
                                animate={{opacity: 1, y: 0}}
                                transition={{delay: 0.3}}
                            >
                                <Card className={"bg-white/5 border border-white/10"}>
                                    <CardHeader>
                                        <div className={"flex items-center gap-3"}>
                                            <Icon icon={"mdi:account-group"} className={"text-2xl text-primary"}/>
                                            <div>
                                                <h3 className={"text-lg font-semibold"}>Created Users</h3>
                                                <p className={"text-sm text-default-500"}>{accountSettings.users.length} user(s) created</p>
                                            </div>
                                        </div>
                                    </CardHeader>
                                    <CardBody>
                                        <div className={"space-y-3"}>
                                            {accountSettings.users.map((user, index) => (
                                                <div key={index} className={"flex items-center justify-between p-3 bg-white/5 rounded-lg border border-white/10"}>
                                                    <div className={"flex items-center gap-3"}>
                                                        <Icon icon={"mdi:account-circle"} className={"text-xl text-default-400"}/>
                                                        <div>
                                                            <div className={"font-medium"}>{user.username}</div>
                                                            <div className={"flex flex-wrap gap-1 mt-1"}>
                                                                {user.permissions.map(permission => (
                                                                    <Chip
                                                                        key={permission}
                                                                        size="sm"
                                                                        variant="flat"
                                                                        color="secondary"
                                                                        className={"text-xs"}
                                                                    >
                                                                        {permission}
                                                                    </Chip>
                                                                ))}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <Button
                                                        size="sm"
                                                        color="danger"
                                                        variant="light"
                                                        onPress={() => handleRemoveUser(user.username)}
                                                        className={"min-w-0 p-2"}
                                                    >
                                                        <Icon icon={"mdi:delete"}/>
                                                    </Button>
                                                </div>
                                            ))}
                                        </div>
                                    </CardBody>
                                </Card>
                            </m.div>
                        )}

                        {/* Setup Summary */}
                        <m.div
                            initial={{opacity: 0, y: 20}}
                            animate={{opacity: 1, y: 0}}
                            transition={{delay: 0.4}}
                        >
                            <Card className={"bg-white/5 border border-white/10"}>
                                <CardHeader>
                                    <div className={"flex items-center gap-3"}>
                                        <Icon icon={"mdi:information"} className={"text-2xl text-info"}/>
                                        <div>
                                            <h3 className={"text-lg font-semibold"}>Setup Summary</h3>
                                            <p className={"text-sm text-default-500"}>Review your account configuration</p>
                                        </div>
                                    </div>
                                </CardHeader>
                                <CardBody>
                                    <div className={"space-y-3 text-sm"}>
                                        <div className={"flex justify-between"}>
                                            <span className={"text-default-500"}>Default admin:</span>
                                            <span className={accountSettings.createDefaultAdmin ? "text-success" : "text-default-400"}>
                                                {accountSettings.createDefaultAdmin ? "Enabled" : "Disabled"}
                                            </span>
                                        </div>
                                        {accountSettings.createDefaultAdmin && (
                                            <div className={"flex justify-between"}>
                                                <span className={"text-default-500"}>Admin username:</span>
                                                <span>{accountSettings.adminUsername}</span>
                                            </div>
                                        )}
                                        <div className={"flex justify-between"}>
                                            <span className={"text-default-500"}>Additional users:</span>
                                            <span>{accountSettings.users.length}</span>
                                        </div>
                                        <div className={"pt-2 border-t border-white/10"}>
                                            <span className={"text-default-500 text-xs"}>
                                                Total accounts: {(accountSettings.createDefaultAdmin ? 1 : 0) + accountSettings.users.length}
                                            </span>
                                        </div>
                                    </div>
                                </CardBody>
                            </Card>
                        </m.div>
                    </div>
                </div>
            </div>
        </m.div>
    );
}