/**
 * Account-related type definitions for Aurora Launcher.
 * Matches the Rust backend's Account and related structs.
 */

/** Account type: Microsoft online or offline mode. */
export type AccountType = 'Microsoft' | 'Offline';

/** Microsoft OAuth device code response (from account:device_code event). */
export interface DeviceCodeResponse {
  user_code: string;
  verification_uri: string;
}

/** Account record matching the Rust backend's Account struct. */
export interface Account {
  id: string;
  username: string;
  display_name: string | null;
  uuid: string | null;
  account_type: AccountType;
  is_active: boolean;
  created_at: string;
}

/** Login state for the account store. */
export type LoginState = 'idle' | 'pending' | 'awaiting_user' | 'success' | 'error';
