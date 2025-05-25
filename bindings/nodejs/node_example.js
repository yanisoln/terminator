const { Desktop } = require('./index.js');

class DesktopAutomation {
    constructor() {
        // Initialize with all features enabled
        this.desktop = Desktop.withAllFeatures();
    }

    async getSystemInfo() {
        const root = this.desktop.root();
        console.log('System Root:', {
            role: root.role(),
            name: root.name(),
            bounds: root.bounds()
        });
    }

    async listApplications() {
        const apps = this.desktop.applications();
        console.log('\nRunning Applications:');
        apps.forEach(app => {
            const attrs = app.attributes();
            console.log(`- ${attrs.name || 'unnamed'} (${attrs.role})`);
            if (app.isFocused()) {
                console.log('  * Currently focused');
            }
        });
    }

    async findAndInteractWithElement(selector) {
        console.log(`\nSearching for element: ${selector}`);
        const locator = this.desktop.locator(selector);
        
        try {
            // Wait for element with timeout
            const element = await locator.wait(5000);
            const attrs = await locator.attributes();
            console.log('Found element:', {
                role: attrs.role,
                name: attrs.name,
                bounds: await locator.bounds()
            });

            // Check if element is visible and enabled
            if (await locator.isVisible()) {
                console.log('Element is visible');
                const enabledElement = await locator.expectEnabled();
                if (enabledElement) {
                    console.log('Element is enabled');
                    
                    // Try to click the element
                    const clickResult = await locator.click();
                    console.log('Click result:', {
                        method: clickResult.method,
                        coordinates: clickResult.coordinates,
                        details: clickResult.details
                    });

                    // Try double click
                    const doubleClickResult = await locator.doubleClick();
                    console.log('Double click result:', {
                        method: doubleClickResult.method,
                        coordinates: doubleClickResult.coordinates,
                        details: doubleClickResult.details
                    });

                    // Try right click
                    await locator.rightClick();
                    console.log('Right click performed');

                    // Try hover
                    await locator.hover();
                    console.log('Hover performed');

                    // Get text content
                    const text = await locator.text();
                    console.log('Element text:', text);

                    // Get bounds
                    const bounds = await locator.bounds();
                    console.log('Element bounds:', bounds);
                }
            }
        } catch (e) {
            if (e && e.code === 'ElementNotFoundError') {
                console.log('Element not found:', e.message);
            } else if (e && e.code === 'TimeoutError') {
                console.log('Timeout waiting for element:', e.message);
            } else {
                console.log('Error interacting with element:', e);
            }
        }
    }

    async captureAndAnalyzeScreen() {
        try {
            // Capture screen
            const screenshot = await this.desktop.captureScreen();
            console.log('\nScreenshot captured:', {
                dimensions: `${screenshot.width}x${screenshot.height}`,
                size: `${screenshot.imageData.length} bytes`
            });

            // Perform OCR on the screenshot
            const text = await this.desktop.ocrScreenshot(screenshot);
            console.log('\nOCR Result:', text);
        } catch (e) {
            console.log('Screenshot/OCR failed:', e);
        }
    }

    async workWithCurrentWindow() {
        try {
            // Get current window
            const window = await this.desktop.getCurrentWindow();
            const windowAttrs = window.attributes();
            console.log('\nCurrent Window:', {
                role: windowAttrs.role,
                name: windowAttrs.name,
                bounds: window.bounds()
            });

            // Get current application
            const app = await this.desktop.getCurrentApplication();
            const appAttrs = app.attributes();
            console.log('Current Application:', {
                role: appAttrs.role,
                name: appAttrs.name
            });

            // Get focused element
            const focused = this.desktop.focusedElement();
            const focusedAttrs = focused.attributes();
            console.log('Focused Element:', {
                role: focusedAttrs.role,
                name: focusedAttrs.name
            });
        } catch (e) {
            if (e && e.code === 'ElementNotFoundError') {
                console.log('No window found - this is normal if no window is focused');
            } else {
                console.log('Error getting window info:', e);
            }
        }
    }

    async runSystemCommand() {
        try {
            // Use cmd.exe /c to properly expand environment variables
            const cmd = await this.desktop.runCommand(
                'cmd.exe /c echo %USERNAME%',  // windows_command
                'echo $USER'                   // unix_command
            );
            console.log('\nCommand Output:', {
                exitStatus: cmd.exitStatus,
                stdout: cmd.stdout.trim(),
                stderr: cmd.stderr
            });
        } catch (e) {
            console.log('Command failed:', e);
        }
    }

    async openAndActivateApp(appName) {
        try {
            console.log(`\nOpening application: ${appName}`);
            this.desktop.openApplication(appName);
            
            // Wait a bit for the app to start
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            console.log(`Activating application: ${appName}`);
            this.desktop.activateApplication(appName);

            // Wait for the application to be fully activated
            await new Promise(resolve => setTimeout(resolve, 1000));
        } catch (e) {
            console.log(`Error with application ${appName}:`, e);
        }
    }
}

async function main() {
    const automation = new DesktopAutomation();
    
    // Get system information
    await automation.getSystemInfo();
    
    // List all running applications
    await automation.listApplications();
    
    // Work with current window and focused elements
    await automation.workWithCurrentWindow();
    
    // Find and interact with elements
    await automation.findAndInteractWithElement('role:button');
    await automation.findAndInteractWithElement('name:Close');
    
    // Capture and analyze screen
    await automation.captureAndAnalyzeScreen();
    
    // Run a system command
    await automation.runSystemCommand();
    
    // Try to open and activate an application
    await automation.openAndActivateApp('notepad');
}

// Run the example
main().catch(err => {
    console.error('Fatal error:', err);
    process.exit(1);
}); 