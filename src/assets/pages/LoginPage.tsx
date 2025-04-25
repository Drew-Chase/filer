import {Button, cn, Form, Image, Input, Switch} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import logo from "../images/filer-logo.svg";
import {useState} from "react";
import {useAuth} from "../providers/AuthProvider.tsx";
import {useNavigate} from "react-router-dom";

export default function LoginPage()
{
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [showPasswordField, setShowPasswordField] = useState(false);
    const [remember, setRemember] = useState(false);
    const [errorMessage, setErrorMessage] = useState<string|null>(null);
    const {login, isLoggedIn} = useAuth();
    const navigate = useNavigate();
    if (isLoggedIn)
    {
        navigate("/files/");
        return null;
    }
    return (
        <div className={"flex flex-col w-1/2 max-w-96 min-w-32 gap-4 mx-auto h-screen items-center justify-center"}>
            <h1 className={"text-4xl font-bold flex flex-row gap-4 items-center"}><span><Image src={logo} width={48}/></span> Filer</h1>
            <Form
                className={"w-full"}
                onSubmit={
                    async (e) =>
                    {
                        e.preventDefault();
                        const error = await login(username, password, remember);
                        if (!error) {
                            setErrorMessage(null);
                            navigate("/files/");
                        } else {
                            console.error("Login failed:", error);
                            setErrorMessage("Invalid username or password.");
                        }
                    }
                }

            >

                <Input
                    label={"Username"}
                    size={"sm"}
                    autoFocus={true}
                    autoComplete={"username"}
                    value={username}
                    onValueChange={setUsername}
                    isRequired
                    endContent={<Icon icon={"mage:user-fill"} className={"opacity-50 my-auto"}/>}
                    classNames={{
                        inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent"
                    }}
                />
                <Input
                    label={"Password"}
                    size={"sm"}
                    value={password}
                    onValueChange={setPassword}
                    isRequired
                    type={showPasswordField ? "text" : "password"}
                    autoComplete={"current-password webauthn"}
                    endContent={
                        <Icon
                            icon={showPasswordField ? "mage:eye-off-fill" : "mage:eye-fill"}
                            className={"opacity-50 data-[active=true]:opacity-100 my-auto cursor-pointer transition-colors duration-200"}
                            onClick={() => setShowPasswordField(prev => !prev)}
                            data-active={showPasswordField ? "true" : "false"}
                        />
                    }
                    classNames={{
                        inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent"
                    }}
                />
                <Switch
                    checked={remember}
                    onValueChange={setRemember}
                    classNames={{
                        base: cn(
                            "flex flex-row-reverse gap-8 items-center justify-between w-full max-w-[unset] p-2 rounded-md",
                            "hover:bg-white/10 duration-200 transition-colors",
                            "data-[selected]:bg-primary/10"
                        )

                    }}
                >
                    <p>Remember Me</p>
                </Switch>
                <Button color={"primary"} fullWidth variant={"ghost"} type={"submit"}>Login</Button>
                <p className={"text-danger underline italic"}>{errorMessage}</p>
            </Form>
        </div>
    );
}
