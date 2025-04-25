import {Button, Link} from "@heroui/react";

export default function ErrorPage()
{
    return (
        <div className={"flex flex-col w-1/2 max-w-96 min-w-32 gap-4 mx-auto h-[calc(100dvh_/_2)] items-center justify-center"}>
            <h1 className={"text-8xl font-bold"}>404</h1>
            <p className={"text-xl"}>Page not found</p>
            <div className={"flex flex-row gap-4 items-center justify-center w-full"}>
                <Button color={"primary"} variant={"ghost"} fullWidth as={Link} href={"/files/"}  >My Files</Button>
                <Button fullWidth as={Link} href={"https://github.com/drew-chase/filer/issues"} target={"_blank"}>Help</Button>
            </div>

        </div>
    );
}
