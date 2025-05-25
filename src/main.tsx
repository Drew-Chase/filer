import React from "react";
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


ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <AuthProvider>
                    <FavoritesProvider>
                        <FileSystemEntryProvider>
                            <MainContentRenderer/>
                        </FileSystemEntryProvider>
                    </FavoritesProvider>
                </AuthProvider>
            </ThemeProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
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
                    <Route path="/" element={<LoginPage/>}/>
                    <Route path={"/files/*"} element={<FilesPage/>}/>
                    <Route path={"/*"} element={<ErrorPage/>}/>
                </Route>
            </Routes>
        </HeroUIProvider>
    );
}
