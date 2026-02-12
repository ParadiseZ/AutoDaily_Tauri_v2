# Policy Editor Implementation Summary

## 1. Overview

The Policy Editor is a complex, nested UI system designed to manage `PolicyInfo`. It allows users to define triggering conditions (vision-based) and execution logic (before/after hooks).

## 2. Key UI Components

- **`PolicyEditor.vue`**: The main container using a **Two-Column Split Layout**.
  - **Left Sidebar**: Metadata editing (Name, Note, Hit Log, Limits).
  - **Right Main Area**: Tabbed interface for `Cond`, `Before`, and `After` logic blocks.
- **`ActionSequenceEditor.vue`**: Manages a linear list of steps.
  - Features a **Categorized Action Picker** (Interaction, Vision, Control, Logic).
  - Supports robust reordering (Move Up/Down) and deletion.
- **`StepItemEditor.vue`**: A collapsible card for individual steps.
  - **Recursive Hosting**: Uses `defineAsyncComponent` to host `ActionSequenceEditor` for container steps (`If`, `While`, `Sequence`), allowing infinite nesting.
  - Visual categorization using gradients and distinct icons.
- **`SearchRuleEditor.vue`**: Recursive editor for complex boolean logic groups (AND/OR/NOT).
  - Supports literal keywords and regular expressions.
  - Includes reactive labels and item counts for nested groups.

## 3. Core Logic & Data Flow

- **Draft Pattern**: New policies are created as local "Drafts" (草案) in `PolicyManagement.vue`. They are only persisted to the database via the sidebar "Save" button.
- **Icon Mapping**: `StepIcon.vue` maps Rust's PascalCase operation strings (e.g., `ClickAction`, `WaitMs`) to Lucide primitives.
- **Reordering**: Step movement is handled via array index swapping in the Vue state, which is reactive and reflects instantly in the UI.

## 4. Backend Integration (Rust)

- **Regex Optimization**: `OcrSearcher` was updated to pre-compile regex patterns during initialization.
- **Standardized OPs**: The frontend uses PascalCase string literals to match the Rust `StepKind` enum variants, ensuring seamless serialization/deserialization.

## 5. Design Aesthetics

- Used **DaisyUI 5.0** and **Tailwind CSS**.
- Implemented modern UI patterns: rounded corners (`rounded-3xl`), glassmorphism, subtle shadows, and connection lines for action sequences.
