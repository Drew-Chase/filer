import {Avatar, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Image, Input, Kbd, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useAuth} from "../providers/AuthProvider.tsx";
import logo from "../images/filer-logo.svg";
import {motion} from "framer-motion";

export default function Navigation()
{
    const {logout, isLoggedIn, username} = useAuth();

    if (!isLoggedIn)
        return null;

    return (
        <motion.div
            initial={{opacity: 0, y: -20}}
            animate={{opacity: 1, y: 0}}
            transition={{duration: .5, delay: 0, type: "spring", ease: "easeInOut"}}
        >
            <Navbar
                maxWidth={"full"}
                className={"bg-transparent backdrop-blur-none backdrop-brightness-100 backdrop-contrast-100 backdrop-saturate-100"}
            >
                <NavbarContent>
                    <NavbarBrand>
                        <Link href={"/files/"} className="font-bold text-inherit flex flex-row items-center gap-2 text-2xl"><Image src={logo} width={32}/></Link>
                    </NavbarBrand>
                </NavbarContent>

                <NavbarContent justify={"center"} className={"w-1/2"}>
                    <Input
                        label={"Search"}
                        placeholder={"Search for files or folders..."}
                        size={"sm"}
                        className={"w-full"}
                        classNames={{
                            inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent"
                        }}
                        endContent={<Kbd keys={["command"]}>K</Kbd>}
                    />
                </NavbarContent>

                <NavbarContent justify="end">
                    {/*<NavbarItem><ThemeSwitchComponent/></NavbarItem>*/}
                    <NavbarItem>
                        <Dropdown
                            classNames={{
                                content: "bg-[#2f115c]/50 outline-primary outline-1 backdrop-brightness-[.5] backdrop-saturate-[1.5] backdrop-blur-lg"
                            }}
                        >
                            <DropdownTrigger><Avatar name={username.toUpperCase()[0]} className={"cursor-pointer"}/></DropdownTrigger>
                            <DropdownMenu
                                itemClasses={{
                                    base: "data-[hover=true]:bg-white/10 data-[selectable=true]:focus:bg-white/50"
                                }}
                            >
                                <DropdownItem key={"profile"} startContent={<Icon icon={"mage:user-fill"}/>}>Profile</DropdownItem>
                                <DropdownItem key={"settings"} startContent={<Icon icon={"mage:settings-fill"}/>}>Settings</DropdownItem>
                                <DropdownItem key={"help"} startContent={<Icon icon={"mage:github"}/>} as={Link} href={"https://github.com/drew-chase/filer/issues"} target={"_blank"}>Feedback / Help</DropdownItem>
                                <DropdownItem key={"logout"} startContent={<Icon icon={"mage:unlocked-fill"}/>} onPress={logout}>Logout</DropdownItem>
                            </DropdownMenu>
                        </Dropdown>
                    </NavbarItem>
                </NavbarContent>
            </Navbar>
        </motion.div>
    );
}
