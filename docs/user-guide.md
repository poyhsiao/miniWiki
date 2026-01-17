# miniWiki User Guide

A comprehensive guide to using miniWiki, your self-hosted knowledge management platform.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Creating Documents](#creating-documents)
3. [Organizing with Spaces](#organizing-with-spaces)
4. [Offline-First Access](#offline-first-access)
5. [Collaboration](#collaboration)
6. [Version History](#version-history)
7. [File Attachments](#file-attachments)
8. [Sharing](#sharing)
9. [Search](#search)
10. [Export](#export)
11. [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Getting Started

### First Login

1. Open miniWiki in your browser
2. Click "Sign Up" to create your account
3. Verify your email address
4. Log in with your credentials

### Dashboard Overview

After logging in, you'll see your dashboard which shows:
- Recent documents you edited
- Spaces you have access to
- Quick actions (New Document, New Space)
- Activity notifications

---

## Creating Documents

### Creating a New Document

1. Click the **+ New** button in the sidebar or use `Ctrl+N` (`Cmd+N` on Mac)
2. Enter a title for your document
3. Start writing using the rich text editor

### Rich Text Formatting

miniWiki uses a block-based editor similar to Notion. Supported formatting:

| Action | Shortcut |
|--------|----------|
| Bold | `Ctrl+B` / `Cmd+B` |
| Italic | `Ctrl+I` / `Cmd+I` |
| Underline | `Ctrl+U` / `Cmd+U` |
| Strikethrough | `Ctrl+Shift+S` |
| Code inline | ``` ` ``` |
| Link | `Ctrl+K` |

### Block Types

Click the `+` icon to add different block types:
- Text paragraph
- Heading (H1, H2, H3)
- Bulleted list
- Numbered list
- Todo list
- Toggle list
- Quote
- Code block
- Divider
- Callout

---

## Organizing with Spaces

### What are Spaces?

Spaces are containers for organizing related documents. Think of them like folders but more flexible.

### Creating a Space

1. Click **Spaces** in the sidebar
2. Click **+ New Space**
3. Enter a name and optional description
4. Choose visibility (Private, Team, or Public)
5. Click **Create**

### Managing Space Members

As a space owner or admin:
1. Open the space
2. Click **Settings** (gear icon)
3. Go to **Members** tab
4. Click **Invite Member**
5. Enter email address and select role:
- **Owner**: Full access, can delete space, manage members, transfer ownership
- **Editor**: Can edit documents, add comments, manage own content
- **Commenter**: Can view documents, add comments, edit own comments
- **Viewer**: Read-only access to documents within the space

### Moving Documents Between Spaces

1. Open the document
2. Click the document title to expand the menu
3. Select **Move to**
4. Choose the destination space

---

## Collaboration

### Real-time Collaboration

When others edit the same document, you'll see:

- **Presence Indicators**: Colored avatars in the header showing who's online
- **Cursor Positions**: Colored cursors with user names tracking where others are editing
- **Live Selections**: Highlighted text showing what others are selecting
- **Typing Indicators**: "User is typing..." notifications

**Connection Status**:
- Green dot: Connected and syncing
- Yellow dot: Reconnecting...
- Red dot: Connection lost (changes saved locally)

### Presence Indicators

1. Select text in a document
2. Click the **Comment** button in the floating toolbar
3. Write your comment
4. Click **Add Comment**
5. Reply directly to threads

### Resolving Comments

- Click **Resolve** to hide a comment thread
- Resolved comments can be viewed in document history

---

## Version History

### Viewing Versions

1. Open a document
2. Click the **三点菜单** (three dots) menu
3. Select **Version History**
4. Browse the list of versions

### Comparing Versions

1. In Version History, select two versions
2. Click **Compare**
3. See differences highlighted

### Restoring Versions

1. Find the version you want to restore
2. Click **Restore**
3. Confirm the action

**Note**: Restoring creates a new version rather than overwriting

---

## File Attachments

### Uploading Files

1. Open a document
2. Click the **📎** (paperclip) button in the toolbar
3. Select a file from your device
4. Click **Upload**

### Supported File Types

- Images: PNG, JPG, GIF, SVG, WebP
- Documents: PDF, DOC, DOCX
- Archives: ZIP
- Videos: MP4, WebM
- Audio: MP3, WAV

### Managing Attachments

- Click an attachment to preview
- Use the context menu to download, rename, or delete
- Drag and drop files directly into documents

---

## Sharing

### Creating Share Links

1. Click the **Share** button in the top right
2. Toggle **Share Link** on
3. Choose permissions:
   - **Can View**: Read-only access
   - **Can Edit**: Full editing access
4. Set expiration (optional)
5. Click **Copy Link**

### Managing Share Links

- View all active links in Share settings
- Revoke links anytime
- See click statistics for each link

### External Access

Share links work for users without miniWiki accounts. They will see a simplified view of the document.

---

## Search

### Quick Search

Press `Ctrl+K` (`Cmd+K` on Mac) to open quick search:
- Search across all accessible documents
- Filter by type (document, space, comment)
- Navigate with arrow keys
- Press Enter to open selected result

### Advanced Search

1. Click the **Search** icon in the sidebar
2. Use filters:
   - Space
   - Date modified
   - Content type
   - Author

### Search Operators

- `"exact phrase"` - Match exact words
- `owner:john` - Documents owned by John
- `space:notes` - Documents in Notes space
- `has:image` - Documents with images
- `modified:>2024-01-01` - Modified after date

---

## Export

### Exporting Documents

1. Open the document
2. Click the **三点菜单** (three dots) menu
3. Select **Export**
4. Choose format:
   - **Markdown** (.md)
   - **HTML** (.html)
   - **PDF** (.pdf)

### Export Options

- Include images
- Preserve formatting
- Add frontmatter (for Markdown)

### Bulk Export

Export entire spaces:
1. Open the space
2. Go to **Settings**
3. Click **Export Space**
4. Choose format and options

---

## Keyboard Shortcuts

### General

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New document |
| `Ctrl+S` | Save (auto-save enabled) |
| `Ctrl+K` | Quick search |
| `Ctrl+/` | Show all shortcuts |
| `Esc` | Close modal/menu |

### Editor

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+U` | Underline |
| `Ctrl+Shift+S` | Strikethrough |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+C` | Copy |
| `Ctrl+V` | Paste |
| `Ctrl+X` | Cut |

### Blocks

| Shortcut | Action |
|----------|--------|
| `/` | Show block menu |
| `Enter` | New block below |
| `Backspace` | Merge with block above |
| `Tab` | Indent / Outdent |

### Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Go to document |
| `Ctrl+G` | Go to line |
| `Alt+←` | Go back |
| `Alt+→` | Go forward |

---

## Offline-First Access

### Working Offline

miniWiki is designed to work seamlessly without an internet connection:

1. **Automatic Offline Mode**
   - When you lose internet connectivity, miniWiki automatically switches to offline mode
   - A status indicator in the header shows your connection status
   - All documents you've recently opened are available for editing

2. **Editing While Offline**
   - Create and edit documents normally
   - Changes are saved locally first
   - The sync indicator shows pending changes (orange dot)
   - All editing features work the same as online

3. **Going Back Online**
   - When connectivity is restored, miniWiki automatically syncs changes
   - A notification shows when sync is complete
   - Conflicts (if any) are resolved automatically using CRDT technology

### Sync Status Indicators

| Indicator | Meaning | Action Required |
|-----------|---------|-----------------|
| 🟢 Green dot | All changes synced | None |
| 🟡 Yellow dot | Syncing in progress | Wait |
| 🟠 Orange dot | Changes pending sync | None (auto-sync) |
| 🔴 Red dot | Sync failed | Check connection |
| ⚪ Gray dot | Offline mode | None (works normally) |

### Managing Local Cache

1. **View Offline Documents**
   - Click the sync indicator
   - See all documents cached for offline use
   - Mark documents for offline availability

2. **Clear Local Data**
   - Go to **Settings > Privacy**
   - Click **Clear Local Cache**
   - Confirm to remove all cached documents

3. **Sync Settings**
   - Automatic sync when online (default)
   - Manual sync option available
   - Sync on app launch toggle

### Conflict Resolution

When the same document is edited offline by multiple users:

1. **Automatic Merging**
   - CRDT technology combines changes intelligently
   - Most conflicts resolve without user input
   - All edits are preserved where possible

2. **Manual Resolution**
   - If automatic resolution isn't possible, you'll be notified
   - Choose which version to keep
   - Compare versions side-by-side

### Tips for Offline Use

1. **Before Going Offline**
   - Open documents you'll need
   - Documents are cached automatically when opened
   - Consider marking important docs for offline access

2. **Battery Considerations**
   - Offline mode uses less battery
   - Background sync is disabled when app is closed
   - Large documents may take longer to sync

3. **Data Limits**
   - Max document size: 10MB
   - Max attachment size: 50MB
   - Cached data stored locally on device

---

## Troubleshooting

### Documents Not Loading

1. Check your internet connection
2. Clear browser cache
3. Try refreshing the page
4. Contact support if issue persists

### Real-time Collaboration Not Working

1. Ensure you're logged in
2. Check WebSocket connection (green dot in header)
3. Try rejoining the document
4. Disable browser extensions that might block WebSockets

### Sync Issues

#### Changes Not Syncing

1. Check your internet connection
2. Look for sync indicator in the header
3. Click the sync indicator and select "Sync Now"
4. Check if document is larger than 10MB limit
5. Try closing and reopening the document

#### Conflict Resolution

1. If you see a conflict notification, click to view details
2. Compare versions side-by-side
3. Select which changes to keep
4. Confirm merge

#### Offline Mode Problems

1. **Can't access documents offline**:
   - Documents must be opened while online first
   - Check sync status for cached documents
   - Try reopening the document while online

2. **Changes lost after going offline**:
   - Verify auto-save was working (check sync status)
   - Look for pending sync items
   - Check if document was within size limits

3. **Sync taking too long**:
   - Large documents take more time
   - Check your internet speed
   - Try syncing during off-peak hours

### Export Fails

1. Large documents may take time to export
2. Check file size limits (max 50MB per file)
3. Try exporting individual sections
4. Clear browser cache and retry

### Mobile App Sync Issues

1. Verify you're on the latest app version
2. Check sync settings
3. Pull down to refresh manually
4. Log out and log back in

---

## Getting Help

### Documentation
- [API Documentation](/api/docs)
- [Developer Guide](/docs/developer.md)
- [Architecture Overview](/docs/architecture.md)

### Community
- [GitHub Issues](https://github.com/kimhsiao/miniWiki/issues)
- [Discussions](https://github.com/kimhsiao/miniWiki/discussions)

### Support
For issues not covered here:
1. Check existing issues on GitHub
2. Create a new issue with:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Browser/OS information
   - Screenshots if relevant

---

## Tips & Best Practices

### Organizing Your Knowledge Base

1. Use consistent naming conventions
2. Create a logical hierarchy with parent/child documents
3. Use tags for cross-space organization
4. Regularly review and archive old content

### Collaboration Etiquette

1. Use comments for discussions instead of modifying directly
2. Communicate changes through @mentions
3. Review version history before making major edits
4. Leave descriptive commit messages for exports

### Performance Optimization

1. Archive old documents instead of deleting
2. Limit document nesting depth (recommended: <5 levels)
3. Use image compression for large files
4. Regular space cleanups

---

*Last updated: January 2026*
