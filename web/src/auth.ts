import { onMount, setContext, getContext } from "svelte";
import { derived, Readable, Writable, writable } from "svelte/store";
import createAuth0Client, { Auth0Client, Auth0ClientOptions } from "@auth0/auth0-spa-js";
import deepmerge from "deepmerge";
import dedupe from "./utils/dedupe-store";
import { error } from "./error";

export const isLoading: Writable<boolean> = writable(true);
export const isAuthenticated: Writable<boolean> = writable(false);
export const authToken: Writable<string> = writable(null);
export const userInfo: Writable<{}> = writable(null);
export const authError: Writable<Error> = writable(null);

const AUTH_KEY = {};

const config: Auth0ClientOptions = {
	domain: import.meta.env.VITE_AUTH0_DOMAIN,
	client_id: import.meta.env.VITE_AUTH0_CLIENT_ID,
	audience: "oil",
	scope: "read:views write:views read:canvas",
	cacheLocation: "localstorage"
};

export type Auth = {
	isLoading: Writable<boolean>;
	isAuthenticated: Writable<boolean>;
	authToken: Writable<string>;
	authError: Writable<Error>;
	login: (opts?: { redirectPage?: string; prompt?: "none" | "login" | "consent" | "select_account" }) => Promise<void>;
	logout: () => void;
	userInfo: Writable<{}>;
};

// Default Auth0 expiration time is 10 hours or something like that.
// If you want to get fancy you can parse the JWT token and get
// token's actual expiration time.
const refreshRate = 10 * 60 * 60 * 1000;

function createAuth() {
	let auth0: Auth0Client = null;
	let intervalId = undefined;

	// You can use Svelte's hooks in plain JS files. How nice!
	onMount(async () => {
		auth0 = await createAuth0Client(config);

		// Not all browsers support this, please program defensively!
		const params = new URLSearchParams(window.location.search);

		// Check if something went wrong during login redirect
		// and extract the error message
		if (params.has("error")) {
			authError.set(new Error(params.get("error_description")));
		}

		// if code then login success
		if (params.has("code")) {
			// Let the Auth0 SDK do it's stuff - save some state, etc.
			await auth0.handleRedirectCallback();
			// Can be smart here and redirect to original path instead of root
			window.history.replaceState({}, document.title, "/");
			authError.set(null);
		}

		const _isAuthenticated = await auth0.isAuthenticated();
		isAuthenticated.set(_isAuthenticated);

		if (_isAuthenticated) {
			// while on it, fetch the user info
			userInfo.set(await auth0.getUser());

			// Get the access token. Make sure to supply audience property
			// in Auth0 config, otherwise you will soon start throwing stuff!
			const token = await auth0.getTokenSilently();
			authToken.set(token);

			if (import.meta.env.DEV) {
				console.log(`auth token: ${token}`);
			}

			// refresh token after specific period or things will stop
			// working. Useful for long-lived apps like dashboards.
			intervalId = setInterval(async () => {
				authToken.set(await auth0.getTokenSilently());
			}, refreshRate);
		}
		isLoading.set(false);

		// clear token refresh interval on component unmount
		return () => {
			intervalId && clearInterval(intervalId);
		};
	});

	// Provide a redirect page if you need.
	// It must be whitelisted in Auth0. I think.
	const login = async (
		opts: {
			redirectPage?: string;
			prompt?: "none" | "login" | "consent" | "select_account";
		} = {}
	) => {
		const { redirectPage, prompt } = opts;

		await auth0.loginWithRedirect({
			redirect_uri: redirectPage ?? window.location.origin,
			prompt: prompt ?? "login"
		});
	};

	const logout = () => {
		// clear local storage, which includes views among other things
		localStorage.clear();

		auth0.logout({
			returnTo: window.location.origin
		});
	};

	const auth: Auth = {
		isLoading,
		isAuthenticated,
		authToken,
		authError,
		login,
		logout,
		userInfo
	};

	// Put everything in context so that child
	// components can access the state
	setContext(AUTH_KEY, auth);

	return auth;
}

// helper function for child components
// to access the auth context
function getAuth(): Auth {
	return getContext(AUTH_KEY);
}

export { createAuth, getAuth };

export const makeAuthedRequest: Readable<(path: string, init?: RequestInit) => Promise<Response>> = derived(
	authToken,
	($authToken) =>
		async (path: string, init?: RequestInit): Promise<Response> => {
			const resp = await fetch(
				`${import.meta.env.VITE_OIL_URL}${path}`,
				deepmerge(
					{
						headers: {
							Authorization: `Bearer ${$authToken}`
						}
					},
					init ?? {}
				)
			);

			switch (resp.status) {
				case 500:
					error.set({ type: "server_error", body: await resp.json() });
					break;

				case 200:
				case 404:
					return resp;

				default:
					error.set({ type: "unknown_http_status", status: resp.status, resp });
			}
		}
);
