import {useEffect, useState} from "react";
import {Button, cn} from "@heroui/react";
import {AnimatePresence, motion} from "framer-motion";
import $ from "jquery";

type DragDropAreaProps = {
    onFileSelected: (files: File[]) => void;
};
export default function DragDropArea(props: DragDropAreaProps)
{
    const [isDragging, setIsDragging] = useState(false);
    const [isUnloadingComponent, setIsUnloadingComponent] = useState(false);

    useEffect(() =>
    {
        // Set up jQuery file drop hover event
        const dropArea = $("#drag-drop-area");

        dropArea.on("dragenter", (e) =>
        {
            e.preventDefault();
            e.stopPropagation();
            setIsDragging(true);
        });

        dropArea.on("dragleave", (e) =>
        {
            e.preventDefault();
            e.stopPropagation();
            setIsDragging(false);
        });

        dropArea.on("dragover", (e) =>
        {
            e.preventDefault();
            e.stopPropagation();
            setIsDragging(true);
        });

        dropArea.on("drop", (e) =>
        {
            e.preventDefault();
            e.stopPropagation();

            const files = e.originalEvent?.dataTransfer?.files;

            if (files && files.length > 0)
            {
                processFiles([...files]);
            }
        });

        // Clean up all event listeners when component unmounts
        return () =>
        {
            dropArea.off("dragenter dragleave dragover drop");
        };
    }, []);

    // Function to process the file - you can implement your logic here
    const processFiles = (filePath: File[]) =>
    {
        setIsUnloadingComponent(true);
        setTimeout(() => props.onFileSelected(filePath), 200);
    };

    return (
        <motion.div
            id={"drag-drop-area"}
            className={
                cn(
                    "flex flex-col items-center justify-center w-full relative",
                    "mt-2 mb-4 p-8 gap-8",
                    "data-[dragover=true]:bg-primary/10 data-[dragover=true]:border-primary"
                )
            }
            animate={{opacity: isUnloadingComponent ? 0 : 1, y: isUnloadingComponent ? 40 : 0}}
            transition={{duration: 0.5, type: "spring"}}
            initial={{opacity: 0, y: 40}}
            exit={{opacity: 0}}
            data-dragover={isDragging}
        >
            <motion.p
                className={"font-bold text-6xl"}
                animate={{
                    scale: isDragging ? 1.2 : 1,
                    y: isDragging ? 40 : 0 // Move down when dragging
                }}
                transition={{
                    type: "spring",
                    stiffness: 300,
                    damping: 20
                }}
            >
                Drag &amp; Drop
            </motion.p>

            <AnimatePresence>
                <motion.div
                    className={"flex flex-row w-full gap-4 items-center justify-center"}
                    initial={{opacity: 1, y: 0}}
                    exit={{opacity: 0, y: 20}}
                    transition={{duration: 0.2}}
                    animate={{y: isDragging ? 40 : 0, opacity: isDragging ? 0 : 1}}
                >
                    <div className={"bg-foreground/25 w-full h-[1px]"}></div>
                    <p>OR</p>
                    <div className={"bg-foreground/25 w-full h-[1px]"}></div>
                </motion.div>
            </AnimatePresence>

            <AnimatePresence>
                <motion.div
                    className="w-full"
                    initial={{opacity: 1, y: 0}}
                    exit={{opacity: 0, y: 20}}
                    transition={{duration: 0.2}}
                    animate={{y: isDragging ? 40 : 0, opacity: isDragging ? 0 : 1}}
                >
                    <Button
                        fullWidth
                        color={"primary"}
                        variant={"solid"}
                        onPress={async () =>
                        {
                            let input = document.createElement("input");
                            input.type = "file";
                            input.multiple = true;

                            input.addEventListener("change", (e: Event) =>
                            {
                                const files = (e.target as HTMLInputElement)?.files;
                                if (files && files.length > 0)
                                {
                                    processFiles([...files]);
                                }
                            });
                            input.click();
                        }}
                    >
                        Select File(s)
                    </Button>
                </motion.div>
            </AnimatePresence>
        </motion.div>
    );
}