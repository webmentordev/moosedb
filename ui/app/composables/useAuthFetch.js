export const useAuthFetch = () => {
    const { getToken } = useAuthToken();
    const authFetch = async (url, options = {}) => {
        const token = getToken();
        console.log(token);
        const defaultOptions = {
            headers: {
                'Authorization': token ? `Bearer ${token}` : '',
                'Accept': 'application/json',
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        };
        try {
            return await $fetch(url, defaultOptions);
        } catch (error) {
            if (error.status === 401) {
                console.log(error);
                const { removeToken } = useAuthToken();
                removeToken();
                await navigateTo('/_/auth/login');
            }
            throw error;
        }
    };
    return { authFetch };
};