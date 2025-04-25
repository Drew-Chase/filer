import {useLocation, useNavigate} from "react-router-dom";
import {useEffect, useState} from "react";
import {useAuth} from "../providers/AuthProvider.tsx";
import {Table} from "@heroui/react";

export default function FilesPage()
{
    const {isLoggedIn} = useAuth();
    const {pathname} = useLocation();
    const [filePath, setFilePath] = useState("");
    const navigate = useNavigate();
    useEffect(() =>
    {
        let path = pathname
            .replace("/files/", "")  // Remove the /files/ prefix
            .replace(/^\//, "");     // Remove leading slash if present
        if (path == "")
            path = "/"
        console.log(path);
        setFilePath(path);
    }, [pathname]);
    useEffect(() =>
    {
        if (!isLoggedIn)
            navigate("/");
    }, [isLoggedIn]);
    return (
        <>
            <h1 className="text-3xl font-bold underline">FilePath is {filePath}</h1>
            <Table>

            </Table>
        </>
    );
}