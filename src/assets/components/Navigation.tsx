import {Avatar, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Image, Input, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@heroui/react";
import {ThemeSwitchComponent} from "../providers/ThemeProvider.tsx";
import {Icon} from "@iconify-icon/react";
import {useAuth} from "../providers/AuthProvider.tsx";
import logo from "../images/filer-logo.svg";

export default function Navigation()
{
    const {logout, isLoggedIn, username} = useAuth();

    if (!isLoggedIn)
        return null;

    return (
        <Navbar maxWidth={"full"}>
            <NavbarContent>
                <NavbarBrand>
                    <p className="font-bold text-inherit flex flex-row items-center gap-2 text-2xl"><Image src={logo} width={32}/></p>
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
                />
            </NavbarContent>

            <NavbarContent justify="end">
                <NavbarItem><ThemeSwitchComponent/></NavbarItem>
                <NavbarItem>
                    <Dropdown classNames={{
                        content: "bg-white/10 backdrop-blur-sm"
                    }}>
                        <DropdownTrigger><Avatar name={username.toUpperCase()[0]} className={"cursor-pointer"}/></DropdownTrigger>
                        <DropdownMenu>
                            <DropdownItem key={"profile"} startContent={<Icon icon={"mage:user-fill"}/>}>Profile</DropdownItem>
                            <DropdownItem key={"settings"} startContent={<Icon icon={"mage:settings-fill"}/>}>Settings</DropdownItem>
                            <DropdownItem key={"help"} startContent={<Icon icon={"mage:github"}/>} as={Link} href={"https://github.com/drew-chase/filer/issues"} target={"_blank"}>Feedback / Help</DropdownItem>
                            <DropdownItem key={"logout"} startContent={<Icon icon={"mage:unlocked-fill"}/>} onPress={logout}>Logout</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                </NavbarItem>
            </NavbarContent>
        </Navbar>);
}