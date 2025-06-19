import React, {useEffect} from "react";
import {BrowserRouter, Route, Routes, useNavigate} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";

import "./assets/scss/index.scss";
import LoginPage from "./assets/pages/LoginPage.tsx";
import Navigation from "./assets/components/Navigation.tsx";
import {ThemeProvider} from "./assets/providers/ThemeProvider.tsx";
import {HeroUIProvider, ToastProvider} from "@heroui/react";
import {AuthProvider} from "./assets/providers/AuthProvider.tsx";
import FilesPage from "./assets/pages/FilesPage.tsx";
import ErrorPage from "./assets/pages/ErrorPage.tsx";
import {FileSystemEntryProvider} from "./assets/providers/FileSystemEntryProvider.tsx";
import {FavoritesProvider} from "./assets/providers/FavoritesProvider.tsx";
import {WindowProvider} from "./assets/providers/WindowProvider.tsx";
import {hasCompletedFirstSetup} from "./assets/ts/first-setup.ts";
import SetupPage from "./assets/pages/SetupPage.tsx";
import {SetupProvider} from "./assets/providers/SetupProvider.tsx";


ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <WindowProvider>
                <ThemeProvider>
                    <AuthProvider>
                        <FavoritesProvider>
                            <FileSystemEntryProvider>
                                <MainContentRenderer/>
                            </FileSystemEntryProvider>
                        </FavoritesProvider>
                    </AuthProvider>
                </ThemeProvider>
            </WindowProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
    const [firstSetup, setFirstSetup] = React.useState<boolean>(false);
    useEffect(() =>
    {
        hasCompletedFirstSetup().then(setFirstSetup);
    }, []);
    return (
        <HeroUIProvider navigate={navigate}>
            <ToastProvider
                placement={"bottom-right"}
                toastProps={{
                    shouldShowTimeoutProgress: true,
                    timeout: 5000
                }}
            />
            <Navigation/>
            <Routes>
                <Route>
                    {!firstSetup ? (<Route path={"*"} element={<SetupProvider><SetupPage/></SetupProvider>}/>) : (<>
                        <Route path="" element={<LoginPage/>}/>
                        <Route path={"files/*"} element={<FilesPage/>}/>
                        <Route path={"*"} element={<ErrorPage/>}/>
                    </>)}
                </Route>
            </Routes>
        </HeroUIProvider>
    );
}
