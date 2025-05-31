# 📊 Excel Copilot

A professional desktop application built with **Tauri + React + TypeScript + Rust** that provides an AI-powered assistant for Microsoft Excel using Google Gemini. The application features real-time Excel window interaction via the Terminator UI automation library, enabling seamless integration between AI capabilities and Excel workflows.

![Excel Copilot](https://img.shields.io/badge/Excel-Copilot-2B579A?style=for-the-badge&logo=microsoft-excel)
![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=tauri)
![Gemini](https://img.shields.io/badge/Google-Gemini%20AI-4285F4?style=for-the-badge&logo=google)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust)

## ✨ Features

### 🎯 **Core Functionality**
- **📁 File Operations**: Create new Excel workbooks or open existing files with native file dialogs
- **🤖 AI Integration**: Chat with Google Gemini to analyze and manipulate Excel data intelligently
- **📋 Real-time Preview**: View Excel content directly within the application interface
- **🔄 Live Automation**: Direct manipulation of Excel windows using advanced UI automation
- **📸 Screenshot Capture**: Capture Excel windows for visual analysis and documentation
- **📄 PDF Analysis**: Attach and analyze PDF documents to extract data for Excel integration

### 🎨 **User Interface**
- **🌟 Modern Design**: Glass morphism UI with backdrop blur effects and professional styling
- **📌 Always-on-top Window**: Maintains visibility while working in Excel for seamless workflow
- **📱 Responsive Layout**: Optimized interface that adapts to different screen sizes
- **🎛️ Real-time Feedback**: Live status updates and operation progress indicators

### ⚙️ **Advanced Capabilities**
- **🔧 Function Calling**: Gemini can directly execute Excel operations through structured tool calls
- **📊 Data Analysis**: AI-powered insights, statistical analysis, and data interpretation
- **🧮 Formula Generation**: Automatic creation and optimization of Excel formulas with error detection
- **🎯 Precision Targeting**: Accurate cell selection and range manipulation
- **🔍 Error Handling**: Comprehensive formula error detection and automatic correction

## 🚀 Quick Start

### Prerequisites

- **Node.js** (v18 or higher) - [Download](https://nodejs.org/)
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **Windows Operating System** (required for Excel automation)
- **Microsoft Excel** (installed and functional)
- **Google Gemini API Key** - [Get your key](https://makersuite.google.com/app/apikey)

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd excel-copilot
   ```

2. **Install frontend dependencies**
   ```bash
   npm install
   ```

3. **Build Rust backend**
   ```bash
   cd src-tauri
   cargo build
   cd ..
   ```

4. **Start development server**
   ```bash
   npm run tauri dev
   ```

## 🔧 Configuration

### Gemini API Setup

1. Obtain your API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Launch the application
3. Click the "⚙️ Config" button in the chat panel
4. Enter your API key and click "Save"
5. The status indicator will show "🤖 Gemini Ready" when configured

### Excel Integration

The application automatically detects and connects to running Excel instances. Ensure Microsoft Excel is installed and running before using automation features.

## 💡 Usage Examples

### Basic Operations

```
User: "Create a new Excel file"
→ Application creates and opens a new workbook

User: "What's the value in cell A1?"
→ Gemini reads and reports the cell value: "Cell A1 contains: 'Sales Data'"

User: "Write 'Revenue' to cell B1"
→ Gemini writes the text and confirms: "Successfully wrote 'Revenue' to cell B1"

User: "Calculate the sum of A1 through A10 in cell C1"
→ Gemini creates the formula: "Set formula '=SUM(A1:A10)' in cell C1. Result: 1250"
```

### Advanced Analytics

```
User: "Analyze the sales data in columns A through D"
→ Gemini reads the range and provides statistical analysis with insights

User: "Create a monthly budget template with formulas"
→ Gemini generates headers, categories, and appropriate formulas

User: "Find and highlight the maximum value in the current sheet"
→ Gemini locates the highest value and provides context
```

### PDF Integration

```
User: [Attaches invoice.pdf] "Extract this invoice data to Excel"
→ Gemini analyzes the PDF and populates Excel with structured data

User: [Attaches report.pdf] "Create a summary table based on this report"
→ Gemini extracts key information and builds an organized table
```

## 🏗️ Architecture

### Frontend (React + TypeScript)
- **Component Architecture**: Modular React components with TypeScript for type safety
- **State Management**: React hooks for efficient state handling and real-time updates
- **UI Framework**: Custom glass morphism design system with responsive breakpoints
- **Real-time Communication**: WebSocket-like communication with Tauri backend

### Backend (Rust + Tauri)
- **Excel Automation**: Terminator library integration for Windows UI automation
- **API Integration**: Asynchronous Gemini API client with retry logic and error handling
- **File System**: Native file operations via Tauri's secure API layer
- **Process Management**: Efficient handling of Excel processes and window detection

### AI Integration
- **Function Calling**: Structured tool definitions enabling Gemini to execute Excel operations
- **Context Management**: Conversation history preservation with clean state separation
- **Error Recovery**: Robust error detection and automatic correction mechanisms
- **Multi-modal Support**: Text and PDF document analysis capabilities

## 📋 Available Commands

### File Operations
- `open_excel_file` - Open Excel file with native dialog
- `create_new_excel` - Create new Excel workbook
- `save_excel_file` - Save current workbook to disk
- `get_excel_content` - Retrieve sheet data for preview

### Excel Automation
- `excel_read_cell` - Read value from specific cell
- `excel_write_cell` - Write value to specific cell
- `excel_read_range` - Read data from cell range
- `excel_set_formula` - Set formula with error checking
- `excel_take_screenshot` - Capture Excel window

### AI Communication
- `setup_gemini_client` - Configure Gemini API client
- `chat_with_gemini` - Send message with tool calling
- `chat_with_gemini_pdf` - Send message with PDF attachments
- `get_chat_history` - Retrieve conversation history
- `clear_chat_history` - Reset conversation state

## 🔍 Technical Implementation

### Excel Automation with Terminator

The application leverages the Terminator library for precise Excel control:

```rust
// Example: Cell value reading with error handling
let excel_window = self.get_excel_window().await?;
let cell_selector = Selector::Name(cell_address.to_string());
let cell_element = excel_window.locator(cell_selector)?.first().await?;
cell_element.click()?;
let value = cell_element.text(3)?;
```

### Gemini Function Calling

Structured function definitions enable AI-driven Excel operations:

```rust
FunctionDeclaration {
    name: "write_excel_cell".to_string(),
    description: "Write a value to a specific cell in Excel".to_string(),
    parameters: json!({
        "type": "object",
        "properties": {
            "cell_address": {
                "type": "string",
                "description": "The cell address in Excel notation (e.g., A1, B2, C10)"
            },
            "value": {
                "type": "string", 
                "description": "The value to write to the cell"
            }
        },
        "required": ["cell_address", "value"]
    }),
}
```

### Error Handling Protocol

Comprehensive error detection and correction for Excel formulas:

1. **Formula Validation**: Check syntax and function names
2. **Reference Verification**: Ensure cell references exist and contain valid data
3. **Result Monitoring**: Detect formula errors (#NAME?, #REF!, #VALUE!, #DIV/0!)
4. **Automatic Correction**: Fix common issues and retry operations
5. **User Feedback**: Provide clear explanations of errors and corrections

## 🎨 User Interface Components

### Status Bar
- Application title and current operation status
- Gemini connection indicator with visual feedback
- Loading states and progress indicators

### File Operations Panel
- New file creation and existing file opening
- Current file display with path information
- Excel interaction testing capabilities

### Excel Preview Panel
- Live sheet content visualization
- Scrollable table interface with cell highlighting
- Multi-sheet navigation and preview

### Chat Interface
- Conversation history with message timestamps
- Tool call visualization showing function execution
- API key configuration and management
- PDF attachment handling with drag-and-drop support

## 🛠️ Development

### Build for Production

```bash
npm run tauri build
```

### Development with Debug Logging

```bash
RUST_LOG=debug npm run tauri dev
```

### Testing Excel Automation

The application includes a built-in test function accessible via the "🧪 Test Excel Interaction" button to verify automation functionality.

### Code Quality Standards

- **Rust**: Follow Clippy recommendations and use `rustfmt` for formatting
- **TypeScript**: Strict TypeScript configuration with comprehensive type checking
- **Testing**: Unit tests for core functionality and integration tests for automation
- **Documentation**: Comprehensive inline documentation and API references

## 🚀 Deployment

### Windows Installer

The build process generates a Windows installer (MSI) for easy distribution:

```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/msi/
```

### System Requirements

- **OS**: Windows 10/11 (64-bit)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Storage**: 500MB available space
- **Excel**: Microsoft Excel 2016 or later

## 🤝 Contributing

We welcome contributions! Please ensure:

### Code Standards
- Follow established architectural patterns
- Include comprehensive error handling
- Write clear, descriptive commit messages
- Add tests for new functionality

### Review Process
- All code must pass CI/CD checks
- Maintain backwards compatibility
- Document breaking changes in commit messages
- Ensure UI accessibility standards

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) file for details.

## 🔮 Roadmap

### Version 2.0
- [ ] **Multi-language Support**: Internationalization with locale-specific formatting
- [ ] **Advanced Charting**: AI-powered chart generation and customization
- [ ] **Template Library**: Pre-built Excel templates for common business scenarios
- [ ] **Collaboration Features**: Real-time sharing and collaborative editing support

### Version 2.5
- [ ] **Voice Commands**: Speech-to-text integration for hands-free operation
- [ ] **OCR Integration**: Image-based data extraction and recognition
- [ ] **Cloud Sync**: Integration with cloud storage providers
- [ ] **Plugin Architecture**: Extensible plugin system for custom functionality

---

**Built with modern technologies for professional Excel automation**

*Excel Copilot combines the power of AI with the precision of automation to transform spreadsheet workflows.*
