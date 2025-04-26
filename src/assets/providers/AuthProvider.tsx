import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {useLocation, useNavigate} from "react-router-dom";

interface AuthContextType
{
    login: (username: string, password: string, remember: boolean) => Promise<string | null>;
    logout: () => void;
    isLoggedIn: boolean;
    username: string;
    isLoading: boolean;
}

interface LoginResponse
{
    token: string;
    username: string;
}

interface ValidateTokenResponse
{
    valid: boolean;
    username?: string;
    error?: string;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({children}: { children: ReactNode })
{
    const navigate = useNavigate();
    const [isLoggedIn, setIsLoggedIn] = useState(true);
    const [username, setUsername] = useState("");
    const [isLoading, setIsLoading] = useState(true);
    const {pathname} = useLocation();

    // Auto-login on component mount
    useEffect(() =>
    {
        const validateToken = async () =>
        {
            try
            {
                const response = await fetch("/api/auth/validate-token", {
                    method: "GET",
                    credentials: "include" // Include cookies in the request
                });

                if (response.ok)
                {
                    const data: ValidateTokenResponse = await response.json();
                    if (data.valid && data.username)
                    {
                        setIsLoggedIn(true);
                        setUsername(data.username);
                        return;
                    }
                }
                throw new Error("Invalid token");
            } catch (error)
            {
                console.error("Auto-login error:", error);

                setIsLoggedIn(false);
                setUsername("");
                navigate("/");
            } finally
            {
                setIsLoading(false);
            }
        };

        validateToken();
    }, []);

    const login = useCallback(async (username: string, password: string, remember: boolean = false) =>
    {
        try
        {
            const response = await fetch("/api/auth/login", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({username, password, remember}),
                credentials: "include" // Important for cookies to be set
            });

            if (!response.ok)
            {
                const errorData = await response.json();
                return errorData.error || "Login failed";
            }

            const data: LoginResponse = await response.json();

            // Update state
            setIsLoggedIn(true);
            setUsername(data.username);

            return null; // Success, no error message
        } catch (error)
        {
            console.error("Login error:", error);
            return "An unexpected error occurred";
        }
    }, []);

    const logout = useCallback(async () =>
    {
        try
        {
            // Call the logout endpoint to clear cookies
            await fetch("/api/auth/logout", {
                method: "POST",
                credentials: "include"
            });
        } catch (error)
        {
            console.error("Logout error:", error);
        } finally
        {
            // Update local state
            setIsLoggedIn(false);
            setUsername("");
            navigate("/");
        }
    }, [navigate]);


    useEffect(() =>
    {
        console.log("Test", isLoggedIn, pathname);
        if (!isLoggedIn && pathname.startsWith("/files/"))
            navigate("/");
        else if (isLoggedIn && !pathname.startsWith("/files/"))
            navigate("/files");
    }, [isLoggedIn, pathname]);

    return (
        <AuthContext.Provider value={{login, logout, isLoggedIn, username, isLoading}}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth(): AuthContextType
{
    const context = useContext(AuthContext);
    if (!context)
    {
        throw new Error("useAuth must be used within a AuthProvider");
    }
    return context;
}