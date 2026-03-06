# Anonymous Progress Sync

This document describes the architecture and behavior of the **Anonymous Progress Sync** feature in Lensisku. This feature allows users who study public flashcard collections without an account to have their progress seamlessly merged into their account once they log in or sign up.

## Overview

The synchronization happens in three phases:
1. **Local Tracking**: While unauthenticated, the frontend tracks a user's study progress using the browser's Local Storage.
2. **Login Trigger**: When the user successfully authenticates, any tracked anonymous progress is extracted and sent to the backend API.
3. **Database Merge (UPSERT)**: The backend securely reads the incoming unauthenticated progress and merges it with any existing authenticated progress the user might already have, utilizing a "Highest Score Wins" strategy.

## 1. Frontend: Tracking Anonymous Progress

When an unauthenticated user interacts with a public collection, the frontend bypasses server-side progress saving and instead uses the `useAnonymousProgress.js` composable.

*   **Storage Mechanism**: Browser `localStorage`
*   **Storage Key**: `lensisku_anon_progress`
*   **Data Structure**:
    ```json
    {
      "collectionId": {
        "levels": {
          "levelId": {
            "cards_completed": 0,
            "correct_answers": 0,
            "total_answers": 0,
            "completed_at": "timestamp"
          }
        }
      }
    }
    ```

In `FlashcardStudyView.vue`, whenever a user submits an answer correctly in anonymous mode, the `saveLevelProgress` function from the composable is called. This function either creates or increments the `cards_completed`, `correct_answers`, and `total_answers` counts for that specific level within that collection in the Local Storage object.

## 2. Frontend: The Login Trigger

The synchronization process is automatically initiated upon a successful login.

*   **Component**: `useAuth.js` composable
*   **Trigger**: The `login()` function

At the end of the `login()` function, an asynchronous call is made to `mergeAnonymousProgressThenClear()`. This function performs the following steps:
1.  **Extraction**: Calls `getAllProgressForMerge()` from the anonymous progress composable. This function formats the local storage data into an array of payloads suitable for the backend API:
    ```json
    [
      {
        "collection_id": 123,
        "level_progress": [
          {
            "level_id": 456,
            "cards_completed": 5,
            "correct_answers": 5,
            "total_answers": 7
          }
        ]
      }
    ]
    ```
2.  **API Call**: Iterates over each collection payload and sends a `POST` request to the backend endpoint: `/api/flashcards/progress/merge`.
3.  **Cleanup**: If the API requests are successful, it calls `clearAfterMerge()` to delete the now-synchronized progress from `localStorage`. This prevents duplicate sync attempts on subsequent logins.

## 3. Backend: The Merge Logic

The backend handles the core merging logic to ensure data integrity and prevent regression of existing progress.

*   **Endpoint**: `POST /api/flashcards/progress/merge`
*   **Handler**: `merge_progress` in `controller.rs` -> `merge_anonymous_progress` in `service.rs`

The merging process in `service.rs` undergoes several steps wrapped within a single database transaction:

### A. Permission Check
The system verifies that the newly authenticated user has read access to the specific collection they are trying to merge progress into (`verify_collection_read_access`). This typically means ensuring the collection is public or owned by the user.

### B. UPSERT and Data Merging
The backend iterates through the incoming progress payloads and executes an `UPSERT` (Insert or Update on Conflict) query against the `user_level_progress` table.

Crucially, **it uses the `GREATEST()` SQL function to resolve conflicts**:

```postgresql
cards_completed = GREATEST(user_level_progress.cards_completed, EXCLUDED.cards_completed),
correct_answers = GREATEST(user_level_progress.correct_answers, EXCLUDED.correct_answers),
total_answers = GREATEST(user_level_progress.total_answers, EXCLUDED.total_answers),
```

**Why this matters:**
*   If the user has **no prior progress** for this level, the anonymous progress is simply inserted.
*   If the user **already has progress** (e.g., they studied on their phone while logged out, but previously studied on their desktop while logged in), the system retains whichever score is higher for each metric. This ensures that a user never loses their best high score or completion count.

### C. Completion and Prerequisite Calculation
Within the same `UPSERT` query and subsequent update queries, the backend dynamically calculates the implications of the newly merged progress:

1.  **Level Completion (`completed_at`)**: The system checks if the new `cards_completed` and success rate (`correct_answers / total_answers`) meet the thresholds defined in the `flashcard_levels` configuration (`min_cards`, `min_success_rate`). If the thresholds are newly met, it sets the `completed_at` timestamp.
2.  **Unlocking Subsequent Levels (`unlocked_at`)**: After updating the core progress metrics, a final query is executed to evaluate if the newly merged progress satisfies the prerequisites for any locked levels. It uses the custom database function `check_level_prerequisites()`. If a prerequisite is now met, the `unlocked_at` timestamp is set for those dependent levels, making them available to the user.
