# miniWiki User Guide

A comprehensive guide to using miniWiki, your self-hosted knowledge management platform.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Creating Documents](#creating-documents)
3. [Organizing with Spaces](#organizing-with-spaces)
4. [Collaboration](#collaboration)
5. [Version History](#version-history)
6. [File Attachments](#file-attachments)
7. [Sharing](#sharing)
8. [Search](#search)
9. [Export](#export)
10. [Keyboard Shortcuts](#keyboard-shortcuts)

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
   - **Owner**: Full access, can delete space
   - **Editor**: Can edit documents
   - **Commenter**: Can view and comment
   - **Viewer**: Read-only access

### Moving Documents Between Spaces

1. Open the document
2. Click the document title to expand the menu
3. Select **Move to**
4. Choose the destination space

---

## Collaboration

### Real-time Collaboration

When others edit the same document, you'll see:
- Their cursor position with name
- Highlighted selections they're working on
- Presence indicators in the header

### Comments

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

*Last updated: January 2025*
