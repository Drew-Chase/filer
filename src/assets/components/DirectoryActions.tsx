import {motion} from "framer-motion";
import {useState} from "react";
import {Icon} from "@iconify-icon/react";
import {addToast, cn, Tooltip} from "@heroui/react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";

export function DirectoryActions()
{
    const {
        selectedEntries,
        downloadSelected,
        askDeleteSelectedEntries,
        refresh,
        askUploadEntry
    } = useFileSystemEntry();

    // Add a state to track if the parent container is being hovered
    const [isHovering, setIsHovering] = useState(false);
    const [alwaysShow, setAlwaysShow] = useState(localStorage.getItem("alwaysShowDirectoryActions") === "true");

    return (
        <motion.div
            className={"flex flex-row gap-2 bg-blue-600/0 w-1/2 justify-end relative py-1"}
            onMouseEnter={() => setIsHovering(true)}
            onMouseLeave={() => setIsHovering(false)}
            onDoubleClick={() =>
            {
                if (sessionStorage.getItem("alwaysShowDirectoryActionNotification") != null)
                {
                    sessionStorage.setItem("alwaysShowDirectoryActionNotification", "true");
                    addToast({
                        title: "Double-click detected",
                        description: "You can double-click to toggle the directory actions menu."
                    });
                }
                setAlwaysShow(prev =>
                {
                    const value = !prev;
                    localStorage.setItem("alwaysShowDirectoryActions", value.toString());
                    return value;
                });
            }}
        >
            <DirectoryActionButton
                count={selectedEntries.size}
                icon={"mage:copy-fill"}
                tooltip={`Copy/Move ${selectedEntries.size} file${selectedEntries.size === 1 ? "" : "s"}`}
                onPress={downloadSelected}
                isPositiveCountRequired={true}
                isVisible={alwaysShow || isHovering}
                index={6}
            />
            <DirectoryActionButton
                count={selectedEntries.size}
                icon={"solar:zip-file-bold"}
                tooltip={`Archive ${selectedEntries.size} file${selectedEntries.size === 1 ? "" : "s"}`}
                onPress={downloadSelected}
                isPositiveCountRequired={true}
                isVisible={alwaysShow || isHovering}
                index={5}
            />
            <DirectoryActionButton
                count={selectedEntries.size}
                icon={"mage:file-download-fill"}
                tooltip={`Download ${selectedEntries.size} file${selectedEntries.size === 1 ? "" : "s"}`}
                onPress={downloadSelected}
                isPositiveCountRequired={true}
                isVisible={alwaysShow || isHovering}
                index={4}
            />
            <DirectoryActionButton
                count={selectedEntries.size}
                icon={"mage:trash-fill"}
                tooltip={`Delete ${selectedEntries.size} file${selectedEntries.size === 1 ? "" : "s"}`}
                onPress={askDeleteSelectedEntries}
                isPositiveCountRequired={true}
                isVisible={alwaysShow || isHovering}
                color={"danger"}
                index={3}
            />

            <DirectoryActionButton
                icon={"mage:file-plus-fill"}
                tooltip={`Create a file or directory`}
                onPress={downloadSelected}
                isPositiveCountRequired={false}
                showCount={false}
                isVisible={alwaysShow || isHovering}
                index={2}
            />

            <DirectoryActionButton
                icon={"iconamoon:cloud-upload-fill"}
                tooltip={`Upload a file or directory`}
                onPress={askUploadEntry}
                isPositiveCountRequired={false}
                showCount={false}
                isVisible={alwaysShow || isHovering}
                index={1}
            />
            <DirectoryActionButton
                icon={"tabler:reload"}
                tooltip={`Refresh current directory`}
                onPress={refresh}
                showCount={false}
                isPositiveCountRequired={false}
                isVisible={alwaysShow || isHovering}
                color={"primary"}
                index={0}
            />
        </motion.div>
    );
}

type DirectoryActionButtonProps = {
    count?: number;
    isPositiveCountRequired?: boolean;
    onPress: () => void;
    icon: string;
    tooltip: string;
    showCount?: boolean;
    isVisible: boolean;
    color?: "primary" | "danger" | "default";
    index: number;
}

function DirectoryActionButton(props: DirectoryActionButtonProps)
{
    const [id] = useState(`directory-action-button-${Math.random().toString(36).substring(2, 15)}`);
    const [hovering, setHovering] = useState(false);
    const isPositiveCountRequired = props.isPositiveCountRequired ?? false;
    const showCount = props.showCount ?? true;
    const color = props.color ?? "default";

    if (props.count === 0 && isPositiveCountRequired) return null;

    return (
        <Tooltip content={props.tooltip} closeDelay={0} delay={1500}>
            <motion.div
                id={id}
                className={
                    cn(
                        "flex items-center justify-center aspect-square w-10 h-10 rounded-full relative",
                        "drop-shadow-md shadow-sm",
                        "data-[hover=true]:brightness-95 data-[hover=true]:text-black cursor-pointer",
                        "data-[color=primary]:bg-primary data-[color=primary]:text-white",
                        "data-[color=danger]:bg-danger data-[color=danger]:text-white",
                        "data-[color=default]:bg-white data-[color=default]:text-black"
                    )
                }
                initial={{
                    x: (props.index === 0 ? 0 : 43 * props.index),
                    scale: (props.index === 0 ? 1 : 0.8),
                    opacity: (props.index === 0 ? 1 : 0.5)
                }}
                animate={{
                    x: props.isVisible ? 0 : (props.index === 0 ? 0 : 43 * props.index),
                    scale: props.isVisible ? 1 : (props.index === 0 ? 1 : 0.8),
                    opacity: props.isVisible ? 1 : (props.index === 0 ? 1 : props.index < 5 ? 0.5 : 0)
                }}
                transition={{
                    duration: 0.3,
                    delay: props.isVisible ? props.index * 0.1 : 0,
                    type: "spring",
                    stiffness: 400,
                    damping: 20
                }}
                onMouseEnter={() => setHovering(true)}
                onMouseLeave={() => setHovering(false)}
                onClick={props.onPress}
                data-hover={hovering}
                data-color={color}
            >
                {showCount && (
                    <motion.div
                        key={props.count}
                        initial={{scale: 1, y: -5}}
                        animate={{scale: 1, y: 0}}
                        transition={{
                            type: "spring",
                            stiffness: 300,
                            damping: 10
                        }}
                        className={
                            cn(
                                "absolute top-0 right-0 -mt-2 -mr-2 z-20 p-1 aspect-square w-6 h-6",
                                "bg-white text-black text-xs rounded-full flex items-center justify-center"
                            )
                        }
                        data-hover={hovering}
                    >
                        {props.count}
                    </motion.div>
                )}
                <Icon icon={props.icon}/>
            </motion.div>
        </Tooltip>
    );
}