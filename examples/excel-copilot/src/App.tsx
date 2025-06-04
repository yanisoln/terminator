import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

interface ToolCall {
  function_name: string;
  arguments: any;
  result: string;
}

interface ChatMessage {
  role: string;
  content: string;
  timestamp: number;
  tool_calls?: ToolCall[];
  response_details?: {
    has_tool_calls: boolean;
    iterations: number;
  };
}

function App() {
  const [currentFile, setCurrentFile] = useState<string>('');
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [currentMessage, setCurrentMessage] = useState('');
  const [isGeminiConfigured, setIsGeminiConfigured] = useState(false);
  const [geminiApiKey, setGeminiApiKey] = useState('');
  const [showApiKeyInput, setShowApiKeyInput] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState('ready');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [attachedPdfs, setAttachedPdfs] = useState<string[]>([]);
  const [copilotEnabled, setCopilotEnabled] = useState(false);
  const [sheetsMode, setSheetsMode] = useState<'excel' | 'googlesheets'>('excel');
  const [googleSheetsStatus, setGoogleSheetsStatus] = useState<string>('');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  useEffect(() => {
    loadChatHistory();
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [chatMessages]);

  // Check if screen is small
  useEffect(() => {
    const checkScreenSize = () => {
      if (window.innerWidth <= 640) {
        // On small screens, start with sidebar collapsed if chat has messages
        if (chatMessages.length > 0) {
          setSidebarCollapsed(true);
        }
      } else {
        setSidebarCollapsed(false);
      }
    };

    checkScreenSize();
    window.addEventListener('resize', checkScreenSize);
    return () => window.removeEventListener('resize', checkScreenSize);
  }, [chatMessages.length]);

  const toggleSidebar = () => {
    setSidebarCollapsed(!sidebarCollapsed);
  };

  const loadChatHistory = async () => {
    try {
      const history = await invoke<ChatMessage[]>('get_chat_history');
      setChatMessages(history);
    } catch (error) {
      console.error('failed to load chat history:', error);
    }
  };

  const showStatus = (message: string) => {
    setStatus(message);
    setTimeout(() => setStatus('ready'), 3000);
  };

  const handleNewFile = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('create_new_excel');
      setCurrentFile('new workbook');
      showStatus(`excel file created: ${result}`);
    } catch (error) {
      console.error('error creating new file:', error);
      showStatus('failed to create new file');
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenFile = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('open_excel_file');
      setCurrentFile(result);
      showStatus('file opened successfully');
    } catch (error) {
      console.error('error opening file:', error);
      showStatus('failed to open file');
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenGoogleSheets = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<string>('open_google_sheets');
      showStatus(`google sheets opened: ${result}`);
      setTimeout(checkGoogleSheetsAvailability, 2000);
    } catch (error) {
      console.error('error opening google sheets:', error);
      showStatus('failed to open google sheets');
    } finally {
      setIsLoading(false);
    }
  };

  const checkGoogleSheetsAvailability = async () => {
    try {
      const result = await invoke<string>('check_google_sheets_availability');
      setGoogleSheetsStatus(result);
      if (result.includes('available and ready')) {
        showStatus('google sheets with gemini is ready');
      } else {
        showStatus('google sheets availability checked');
      }
    } catch (error) {
      console.error('error checking google sheets:', error);
      setGoogleSheetsStatus('Error checking availability');
    }
  };

  useEffect(() => {
    if (sheetsMode === 'googlesheets') {
      checkGoogleSheetsAvailability();
    }
  }, [sheetsMode]);

  const setupGemini = async () => {
    if (!geminiApiKey.trim()) return;
    
    setIsLoading(true);
    try {
      await invoke('setup_gemini_client', { 
        apiKey: geminiApiKey, 
        copilotEnabled: copilotEnabled 
      });
      setIsGeminiConfigured(true);
      setShowApiKeyInput(false);
      showStatus('gemini configured successfully');
    } catch (error) {
      console.error('error setting up gemini:', error);
      showStatus('failed to configure gemini');
    } finally {
      setIsLoading(false);
    }
  };

  const sendMessage = async () => {
    if (!currentMessage.trim() || !isGeminiConfigured || isLoading) return;

    setIsLoading(true);
    const userMessage = currentMessage;
    const pdfsToSend = [...attachedPdfs];
    setCurrentMessage('');
    setAttachedPdfs([]);

    const newUserMessage: ChatMessage = {
      role: 'user',
      content: userMessage,
      timestamp: Date.now()
    };
    setChatMessages(prev => [...prev, newUserMessage]);

    const assistantPlaceholder: ChatMessage = {
      role: 'model',
      content: 'thinking...',
      timestamp: Date.now(),
      tool_calls: [],
      response_details: {
        has_tool_calls: false,
        iterations: 0
      }
    };
    setChatMessages(prev => [...prev, assistantPlaceholder]);

    try {
      const pollForUpdates = setInterval(async () => {
        try {
          const history = await invoke<ChatMessage[]>('get_chat_history');
          setChatMessages(history);
        } catch (error) {
          console.error('error polling chat history:', error);
        }
      }, 200);

      const response = pdfsToSend.length > 0 
        ? await invoke<string>('chat_with_gemini_pdf', { message: userMessage, pdfFiles: pdfsToSend })
        : await invoke<string>('chat_with_gemini', { message: userMessage });
      
      clearInterval(pollForUpdates);
      await loadChatHistory();
      
      showStatus(`message sent: ${response}`);
    } catch (error) {
      console.error('error sending message:', error);
      showStatus(`error: ${error}`);
      setChatMessages(prev => prev.slice(0, -1));
    } finally {
      setIsLoading(false);
    }
  };

  const selectPdfFiles = async () => {
    try {
      const selectedFiles = await invoke<string[]>('select_pdf_files');
      if (selectedFiles.length > 0) {
        setAttachedPdfs(prev => [...prev, ...selectedFiles]);
        showStatus(`${selectedFiles.length} pdf(s) attached`);
      }
    } catch (error) {
      console.error('error selecting pdf files:', error);
      showStatus('failed to select pdf files');
    }
  };

  const removePdf = (index: number) => {
    setAttachedPdfs(prev => prev.filter((_, i) => i !== index));
    showStatus('pdf removed');
  };

  const testExcelInteraction = async () => {
    setIsLoading(true);
    try {
      showStatus('testing excel interaction...');
      
      const cellValue = await invoke<string>('excel_read_cell', { cellAddress: 'A1' });
      showStatus(`cell a1: ${cellValue}`);
      
      await invoke<string>('excel_write_cell', { 
        cellAddress: 'B1', 
        value: 'hello from copilot!' 
      });
      
      showStatus('excel interaction test completed');
    } catch (error) {
      console.error('excel interaction test failed:', error);
      showStatus(`excel test failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const showLocaleInfo = async () => {
    setIsLoading(true);
    try {
      const localeInfo = await invoke<string>('get_locale_info');
      showStatus(`locale info: ${localeInfo}`);
      
      // Add a message to chat to show locale info
      const userMessage: ChatMessage = {
        role: 'user',
        content: '🌍 System Locale Information Request',
        timestamp: Date.now()
      };
      
      const assistantMessage: ChatMessage = {
        role: 'model',
        content: `📍 **System Locale Information:**\n\n${localeInfo}\n\n*This affects how numbers are formatted when writing to Excel. The system automatically normalizes number formats according to your locale before sending them to Excel.*`,
        timestamp: Date.now()
      };
      
      setChatMessages(prev => [...prev, userMessage, assistantMessage]);
      
    } catch (error) {
      console.error('failed to get locale info:', error);
      showStatus(`locale info failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const clearChat = async () => {
    try {
      await invoke('clear_chat_history');
      setChatMessages([]);
      showStatus('chat cleared');
    } catch (error) {
      console.error('error clearing chat:', error);
      showStatus('failed to clear chat');
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  return (
    <div className="app">
      {/* Header */}
      <header className="header">
        <div className="header-content">
          <div className="logo">
            <div className="logo-icon">E</div>
            <span className="logo-text">excel copilot</span>
          </div>
          <div className="status">
            <span className="status-text">{status}</span>
            {isLoading && <div className="loading-spinner"></div>}
          </div>
          <div className="gemini-indicator">
            <div className={`indicator-dot ${isGeminiConfigured ? 'connected' : 'disconnected'}`}></div>
            <span className="indicator-text">
              {isGeminiConfigured ? 'gemini ready' : 'configure gemini'}
            </span>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="main">
        {/* Sidebar Toggle Button for Mobile */}
        <button 
          className="sidebar-toggle"
          onClick={toggleSidebar}
          title={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
        >
          {sidebarCollapsed ? '☰' : '×'}
        </button>

        {/* Sidebar */}
        <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
          <section className="file-section">
            <h3 className="section-title">spreadsheet operations</h3>
            <div className="app-mode-selector">
              <div className="mode-option">
                <input
                  type="radio"
                  id="excel-mode"
                  name="sheetsMode"
                  value="excel"
                  checked={sheetsMode === 'excel'}
                  onChange={(e) => setSheetsMode(e.target.value as 'excel' | 'googlesheets')}
                  className="mode-radio"
                />
                <label htmlFor="excel-mode" className="mode-label">
                  <div className="mode-icon">📊</div>
                  <div className="mode-info">
                    <div className="mode-title">Microsoft Excel</div>
                    <div className="mode-subtitle">Local files & automation</div>
                  </div>
                </label>
              </div>
              <div className="mode-option">
                <input
                  type="radio"
                  id="sheets-mode"
                  name="sheetsMode"
                  value="googlesheets"
                  checked={sheetsMode === 'googlesheets'}
                  onChange={(e) => setSheetsMode(e.target.value as 'excel' | 'googlesheets')}
                  className="mode-radio"
                />
                <label htmlFor="sheets-mode" className="mode-label">
                  <div className="mode-icon">🌐</div>
                  <div className="mode-info">
                    <div className="mode-title">Google Sheets</div>
                    <div className="mode-subtitle">Browser-based with Gemini</div>
                  </div>
                </label>
              </div>
            </div>
            
            <div className="button-group">
              {sheetsMode === 'excel' ? (
                <>
                  <button 
                    className="btn btn-primary" 
                    onClick={handleNewFile}
                    disabled={isLoading}
                  >
                    new excel file
                  </button>
                  <button 
                    className="btn btn-secondary" 
                    onClick={handleOpenFile}
                    disabled={isLoading}
                  >
                    open excel file
                  </button>
                  <button 
                    className="btn btn-outline" 
                    onClick={testExcelInteraction}
                    disabled={isLoading || !currentFile}
                  >
                    test excel
                  </button>
                </>
              ) : (
                <>
                  <button 
                    className="btn btn-primary" 
                    onClick={handleOpenGoogleSheets}
                    disabled={isLoading}
                  >
                    open new google sheets
                  </button>
                  <div className="google-sheets-info">
                    <div className="info-header">📋 Setup Required:</div>
                    <div className="info-steps">
                      <div className="info-step">1. Open Google Sheets manually OR click button above</div>
                      <div className="info-step">2. Open the document you want to edit, or create a new one</div>
                      <div className="info-step">3. Ensure Gemini is available in your Google Sheets</div>
                      <div className="info-step">4. Start chatting - I'll interact with Gemini for you</div>
                    </div>
                    <div className="info-header" style={{ marginTop: '12px' }}>💡 Tips for Best Results:</div>
                    <div className="info-steps">
                      <div className="info-step">• Be specific: "Create a table with Name, Age, City columns"</div>
                      <div className="info-step">• One task at a time: "Add this data" then "Format as currency"</div>
                      <div className="info-step">• Use simple language: "Make row 1 bold" vs complex formatting</div>
                    </div>
                    {googleSheetsStatus && (
                      <div className={`availability-status ${googleSheetsStatus.includes('available and ready') ? 'ready' : 'warning'}`}>
                        🔍 Status: {googleSheetsStatus}
                      </div>
                    )}
                    <button 
                      className="btn btn-outline btn-small" 
                      onClick={checkGoogleSheetsAvailability}
                      disabled={isLoading}
                      style={{ marginTop: '12px' }}
                    >
                      check availability
                    </button>
                  </div>
                </>
              )}
              
              <button 
                className="btn btn-outline" 
                onClick={showLocaleInfo}
                disabled={isLoading}
                title="show system locale and number formatting info"
              >
                locale info
              </button>
            </div>
            {currentFile && sheetsMode === 'excel' && (
              <div className="current-file">
                <span className="file-label">current file:</span>
                <span className="file-name">{currentFile}</span>
              </div>
            )}
          </section>

          {sheetsMode === 'excel' && (
            <section className="copilot-section">
              <h3 className="section-title">excel copilot settings</h3>
              <div className="copilot-toggle">
                <label className="toggle-label">
                  <input
                    type="checkbox"
                    checked={copilotEnabled}
                    onChange={(e) => setCopilotEnabled(e.target.checked)}
                    className="toggle-input"
                  />
                  <span className="toggle-slider"></span>
                  <span className="toggle-text">Enable MS Excel Copilot</span>
                </label>
              </div>
              
              {copilotEnabled && (
                <div className="copilot-requirements">
                  <div className="requirements-header">⚠️ Requirements:</div>
                  <ul className="requirements-list">
                    <li>📁 File must be saved in <strong>OneDrive</strong></li>
                    <li>📂 Use <strong>"Open"</strong> (not "New") to select existing OneDrive file</li>
                    <li>📊 Data range needs <strong>≥3 rows, ≥2 columns</strong></li>
                    <li>🏷️ <strong>Headers required</strong> in first row</li>
                  </ul>
                  <div className="requirements-note">
                    💡 Copilot only works with OneDrive documents that have properly structured data with headers.
                  </div>
                </div>
              )}
              
              {copilotEnabled && !currentFile.includes('OneDrive') && currentFile && (
                <div className="copilot-warning">
                  ⚠️ Current file may not be in OneDrive. Copilot features might not work.
                </div>
              )}
            </section>
          )}

          {sheetsMode === 'googlesheets' && (
            <section className="google-sheets-section">
              <h3 className="section-title">google sheets requirements</h3>
              <div className="google-sheets-details">
                <div className="requirement-item">
                  <span className="requirement-icon">🔑</span>
                  <div className="requirement-content">
                    <div className="requirement-title">Gemini Access Required</div>
                    <div className="requirement-text">You need access to Gemini in Google Sheets. Look for the "Ask Gemini" button in your Google Sheets interface.</div>
                  </div>
                </div>
                <div className="requirement-item">
                  <span className="requirement-icon">🌐</span>
                  <div className="requirement-content">
                    <div className="requirement-title">Browser-Based</div>
                    <div className="requirement-text">All operations are performed through Google Sheets' built-in Gemini interface via browser automation.</div>
                  </div>
                </div>
                <div className="requirement-item">
                  <span className="requirement-icon">🤖</span>
                  <div className="requirement-content">
                    <div className="requirement-title">AI-Powered Automation</div>
                    <div className="requirement-text">I'll send requests to Google Sheets Gemini and automatically apply the responses for you.</div>
                  </div>
                </div>
              </div>
            </section>
          )}
        </aside>

        {/* Chat Area */}
        <section className={`chat-section ${sidebarCollapsed ? 'expanded' : ''}`}>
          <div className="chat-header">
            <h2 className="chat-title">
              chat with gemini - {sheetsMode === 'excel' ? 'excel mode' : 'google sheets mode'}
            </h2>
            <div className="chat-controls">
              <button 
                className="btn-icon"
                onClick={() => setShowApiKeyInput(!showApiKeyInput)}
                title="configure api key"
              >
                ⚙
              </button>
              <button 
                className="btn-icon"
                onClick={clearChat}
                disabled={chatMessages.length === 0}
                title="clear chat"
              >
                ×
              </button>
            </div>
          </div>

          {showApiKeyInput && (
            <div className="api-key-input">
              <input
                type="password"
                placeholder="enter gemini api key"
                value={geminiApiKey}
                onChange={(e) => setGeminiApiKey(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && setupGemini()}
                className="api-input"
              />
              <button onClick={setupGemini} disabled={isLoading} className="btn btn-primary">
                save
              </button>
            </div>
          )}

          <div className="chat-messages">
            {chatMessages.length === 0 ? (
              <div className="empty-chat">
                <div className="welcome-content">
                  <h3>welcome to excel copilot</h3>
                  <p>connect with gemini to start analyzing your {sheetsMode === 'excel' ? 'excel' : 'google sheets'} data.</p>
                  <ul className="feature-list">
                    <li>ask questions about your data</li>
                    <li>request summaries and insights</li>
                    <li>get help with formulas</li>
                    <li>auto-generate content</li>
                    {sheetsMode === 'googlesheets' && <li>interact via google sheets gemini</li>}
                  </ul>
                  <div className="example-queries">
                    {sheetsMode === 'excel' ? (
                      <>
                        <div className="query-example">"what's the sum of column a?"</div>
                        <div className="query-example">"create a summary of this data"</div>
                        <div className="query-example">"add a formula to calculate average"</div>
                      </>
                    ) : (
                      <>
                        <div className="query-example">"Add this data to the sheet: Name, Age, City"</div>
                        <div className="query-example">"Create a bar chart from columns A to C"</div>
                        <div className="query-example">"Format column B as currency and make row 1 bold"</div>
                      </>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              chatMessages.map((msg, index) => (
                <div key={index} className={`message ${msg.role}-message`}>
                  <div className="message-content">
                    {msg.content}
                  </div>
                  
                  {msg.tool_calls && msg.tool_calls.length > 0 && (
                    <div className="tool-calls">
                      <div className="tool-calls-header">
                        functions called ({msg.tool_calls.length})
                      </div>
                      {msg.tool_calls.map((toolCall, tcIndex) => (
                        <div key={tcIndex} className="tool-call">
                          <div className="tool-call-name">
                            {toolCall.function_name}
                          </div>
                          <div className="tool-call-args">
                            <strong>arguments:</strong>
                            <pre>{JSON.stringify(toolCall.arguments, null, 2)}</pre>
                          </div>
                          <div className="tool-call-result">
                            <strong>result:</strong>
                            <div className="result-content">{toolCall.result}</div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                  
                  {msg.role === 'model' && msg.response_details && (
                    <div className="response-details">
                      {msg.response_details.has_tool_calls && (
                        <span className="detail-badge">
                          used {msg.response_details.iterations} iteration(s)
                        </span>
                      )}
                      <span className="detail-badge">
                        llm response
                      </span>
                    </div>
                  )}
                  
                  <div className="message-time">
                    {formatTime(msg.timestamp)}
                  </div>
                </div>
              ))
            )}
            <div ref={messagesEndRef} />
          </div>

          {isGeminiConfigured && (
            <div className="chat-input">
              {attachedPdfs.length > 0 && (
                <div className="pdf-attachments">
                  <div className="attachments-header">
                    attached pdf files ({attachedPdfs.length})
                  </div>
                  <div className="attachments-list">
                    {attachedPdfs.map((pdf, index) => (
                      <div key={index} className="attachment-item">
                        <span className="attachment-name">{pdf}</span>
                        <button 
                          className="remove-attachment"
                          onClick={() => removePdf(index)}
                          title="remove this pdf"
                        >
                          ×
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              
              <div className="input-controls">
                <button 
                  className="attach-btn"
                  onClick={selectPdfFiles}
                  disabled={isLoading}
                  title="attach pdf files"
                >
                  📎
                </button>
                <input
                  type="text"
                  placeholder={`ask me anything about your ${sheetsMode === 'excel' ? 'excel' : 'google sheets'} data...`}
                  value={currentMessage}
                  onChange={(e) => setCurrentMessage(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
                  disabled={isLoading}
                  className="message-input"
                />
                <button 
                  className="send-btn"
                  onClick={sendMessage}
                  disabled={isLoading || !currentMessage.trim()}
                >
                  {isLoading ? '⏳' : '→'}
                </button>
              </div>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;
