import {Button, cn, Form, Image, Switch} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import logo from "../images/filer-logo.svg";
import {useState} from "react";
import {useAuth} from "../providers/AuthProvider.tsx";
import {useNavigate} from "react-router-dom";
import {motion} from "framer-motion";
import Input from "../components/overrides/Input.tsx";

export default function LoginPage()
{
    const [isUnloading, setIsUnloading] = useState(false);
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [showPasswordField, setShowPasswordField] = useState(false);
    const [remember, setRemember] = useState(false);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const {login} = useAuth();
    const navigateFunction = useNavigate();
    const animationDuration = 0.5;

    return (
        <div className={"flex flex-col w-1/2 max-w-96 min-w-32 gap-4 mx-auto h-screen items-center justify-center"}>
            <motion.div
                className="backdrop-blur-md bg-white/10 rounded-xl shadow-xl p-6 border border-white/20 w-full gap-4 flex flex-col items-center justify-center"
                initial={{opacity: 0}}
                animate={{opacity: isUnloading ? 0 : 1}}
                transition={{duration: animationDuration, delay: 0, type: "spring", ease: "easeInOut"}}
            >
                <motion.div
                    className={"flex flex-row items-center gap-4 text-4xl font-bold"}
                    initial={{opacity: 0, y: 50}}
                    animate={{opacity: isUnloading ? 0 : 1, y: isUnloading ? -50 : 0}}
                    transition={{duration: animationDuration, delay: 0, type: "spring", ease: "easeInOut"}}
                >
                    <span><Image src={logo} width={48}/></span> Filer
                </motion.div>
                <Form
                    className={"w-full"}
                    onSubmit={
                        async (e) =>
                        {
                            e.preventDefault();
                            setIsUnloading(true);
                            const error = await login(username, password, remember);
                            if (!error)
                            {
                                setTimeout(() =>
                                {
                                    setErrorMessage(null);
                                    navigateFunction("/files/");
                                }, 200);
                            } else
                            {
                                console.error("Login failed:", error);
                                setErrorMessage("Invalid username or password.");
                                setIsUnloading(false);
                            }
                        }
                    }
                >

                    <motion.div
                        initial={{opacity: 0, y: 50}}
                        animate={{opacity: isUnloading ? 0 : 1, y: isUnloading ? -50 : 0}}
                        transition={{duration: animationDuration, delay: .1, type: "spring", ease: "easeInOut"}}
                        className={"w-full"}
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
                                inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 data-[focus]:outline-primary outline-2 outline-transparent",
                                input: ["focus:outline-none"]
                            }}
                        />
                    </motion.div>

                    <motion.div
                        initial={{opacity: 0, y: 50}}
                        animate={{opacity: isUnloading ? 0 : 1, y: isUnloading ? -50 : 0}}
                        transition={{duration: animationDuration, delay: .2, type: "spring", ease: "easeInOut"}}
                        className={"w-full"}
                    >
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
                                inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 data-[focus]:outline-primary outline-2 outline-transparent"
                            }}
                        />
                    </motion.div>

                    <motion.div
                        initial={{opacity: 0, y: 50}}
                        animate={{opacity: isUnloading ? 0 : 1, y: isUnloading ? -50 : 0}}
                        transition={{duration: animationDuration, delay: .3, type: "spring", ease: "easeInOut"}}
                        className={"w-full"}
                    >
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
                    </motion.div>

                    <motion.div
                        initial={{opacity: 0, y: 50}}
                        animate={{opacity: isUnloading ? 0 : 1, y: isUnloading ? -50 : 0}}
                        transition={{duration: animationDuration, delay: .4, type: "spring", ease: "easeInOut"}}
                        className={"w-full"}
                    >
                        <Button color={"primary"} fullWidth variant={"ghost"} type={"submit"}>Login</Button>
                    </motion.div>
                    <p className={"text-danger underline italic"}>{errorMessage}</p>
                </Form>
            </motion.div>
        </div>
    );
}
