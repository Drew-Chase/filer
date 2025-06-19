import {m} from "framer-motion";
import {Button, Card, CardBody, Divider, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {QRCodeSVG} from "qrcode.react";

export default function FinishStep()
{
    // Replace these URLs with your actual app download links
    const downloadLinks = {
        android: "https://play.google.com/store/apps/details?id=com.yourapp.filer",
        ios: "https://apps.apple.com/app/filer/id123456789",
        windows: "https://github.com/Drew-Chase/filer/releases/download/latest/filer-windows-setup.exe",
        mac: "https://github.com/Drew-Chase/filer/releases/download/latest/filer-macos.dmg",
        linux: "https://github.com/Drew-Chase/filer/releases/download/latest/filer-linux.deb"
    };

    const handleDownload = (platform: string, url: string) =>
    {
        // Create a temporary anchor element to trigger download
        const link = document.createElement("a");
        link.href = url;
        link.download = `filer-${platform}`;
        link.click();
    };

    return (
        <m.div
            key="finish"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl p-6 border border-white/20 overflow-y-scroll relative"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            <div className={"w-full flex flex-col gap-8 items-center justify-center mt-8"}>
                <div className="text-center">
                    <h1 className={"text-7xl font-bold mb-4"}>Setup Complete!</h1>
                    <p className="text-xl text-gray-400 mb-8">Download our apps to access your files anywhere</p>
                </div>

                {/* Mobile Apps Section */}
                <div className="w-full max-w-6xl">
                    <h2 className="text-3xl font-semibold text-center mb-6">Mobile Apps</h2>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                        {/* Android QR Code */}
                        <Card className="bg-white/10 border border-white/20">
                            <CardBody className="flex flex-col items-center p-8">
                                <div className="flex items-center gap-3 mb-4">
                                    <Icon icon="logos:android-icon" className="text-4xl"/>
                                    <h3 className="text-2xl font-semibold">Android</h3>
                                </div>
                                <div className="bg-white p-4 rounded-lg mb-4">
                                    <QRCodeSVG
                                        value={downloadLinks.android}
                                        size={150}
                                        bgColor="#ffffff"
                                        fgColor="#000000"
                                        level="M"
                                        imageSettings={{
                                            src: "https://raw.githubusercontent.com/Drew-Chase/filer/refs/heads/master/src/assets/images/filer-logo.svg",
                                            x: 150 / 2 - 15, // Center the logo
                                            y: 150 / 2 - 15, // Center the logo
                                            width: 30,
                                            height: 30,
                                            excavate: true
                                        }}

                                    />
                                </div>
                                <p className="text-sm text-gray-400 text-center mb-4">
                                    Scan with your Android device
                                </p>
                                <Button
                                    color="success"
                                    variant="solid"
                                    size="lg"
                                    className="w-full"
                                    as={Link}
                                    href={downloadLinks.android}
                                    target="_blank"
                                    startContent={<Icon icon="logos:google-play-icon"/>}
                                >
                                    Google Play Store
                                </Button>
                            </CardBody>
                        </Card>

                        {/* iOS QR Code */}
                        <Card className="bg-white/10 border border-white/20">
                            <CardBody className="flex flex-col items-center p-8">
                                <div className="flex items-center gap-3 mb-4">
                                    <Icon icon="logos:apple" className="text-4xl"/>
                                    <h3 className="text-2xl font-semibold">iOS</h3>
                                </div>
                                <div className="bg-white p-4 rounded-lg mb-4">
                                    <QRCodeSVG
                                        value={downloadLinks.ios}
                                        size={150}
                                        bgColor="#ffffff"
                                        fgColor="#000000"
                                        level="M"
                                        imageSettings={{
                                            src: "https://raw.githubusercontent.com/Drew-Chase/filer/refs/heads/master/src/assets/images/filer-logo.svg",
                                            x: 150 / 2 - 15, // Center the logo
                                            y: 150 / 2 - 15, // Center the logo
                                            width: 30,
                                            height: 30,
                                            excavate: true
                                        }}
                                    />
                                </div>
                                <p className="text-sm text-gray-400 text-center mb-4">
                                    Scan with your iPhone or iPad
                                </p>
                                <Button
                                    color="primary"
                                    variant="solid"
                                    size="lg"
                                    className="w-full"
                                    as={Link}
                                    href={downloadLinks.ios}
                                    target="_blank"
                                    startContent={<Icon icon="simple-icons:appstore"/>}
                                >
                                    App Store
                                </Button>
                            </CardBody>
                        </Card>
                    </div>
                </div>

                <Divider className="my-4"/>

                {/* Desktop Apps Section */}
                <div className="w-full max-w-6xl">
                    <h2 className="text-3xl font-semibold text-center mb-6">Desktop Apps</h2>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        {/* Windows */}
                        <Card className="bg-white/10 border border-white/20 hover:bg-white/15 transition-colors">
                            <CardBody className="flex flex-col items-center p-6">
                                <Icon icon="logos:microsoft-windows" className="text-6xl mb-4"/>
                                <h3 className="text-xl font-semibold mb-4">Windows</h3>
                                <Button
                                    color="primary"
                                    variant="solid"
                                    size="lg"
                                    className="w-full"
                                    onPress={() => handleDownload("windows", downloadLinks.windows)}
                                    startContent={<Icon icon="material-symbols:download"/>}
                                >
                                    Download for Windows
                                </Button>
                            </CardBody>
                        </Card>

                        {/* macOS */}
                        <Card className="bg-white/10 border border-white/20 hover:bg-white/15 transition-colors">
                            <CardBody className="flex flex-col items-center p-6">
                                <Icon icon="logos:apple" className="text-6xl mb-4"/>
                                <h3 className="text-xl font-semibold mb-4">macOS</h3>
                                <Button
                                    color="secondary"
                                    variant="solid"
                                    size="lg"
                                    className="w-full"
                                    onPress={() => handleDownload("mac", downloadLinks.mac)}
                                    startContent={<Icon icon="material-symbols:download"/>}
                                >
                                    Download for Mac
                                </Button>
                            </CardBody>
                        </Card>

                        {/* Linux */}
                        <Card className="bg-white/10 border border-white/20 hover:bg-white/15 transition-colors">
                            <CardBody className="flex flex-col items-center p-6">
                                <Icon icon="logos:linux-tux" className="text-6xl mb-4"/>
                                <h3 className="text-xl font-semibold mb-4">Linux</h3>
                                <Button
                                    color="warning"
                                    variant="solid"
                                    size="lg"
                                    className="w-full"
                                    onPress={() => handleDownload("linux", downloadLinks.linux)}
                                    startContent={<Icon icon="material-symbols:download"/>}
                                >
                                    Download for Linux
                                </Button>
                            </CardBody>
                        </Card>
                    </div>
                </div>

                <div className="text-center mt-8">
                    <p className="text-sm text-gray-400">
                        Need help? Check our{" "}
                        <Link
                            href="https://github.com/Drew-Chase/filer/wiki"
                            target="_blank"
                            className="text-primary hover:underline"
                        >
                            documentation
                        </Link>
                    </p>
                </div>
            </div>
        </m.div>
    );
}