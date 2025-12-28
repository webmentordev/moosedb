export const useAuthFetch = () => {
    const { getToken } = useAuthToken();
    const authFetch = async (url, options = {}) => {
        const token = getToken();
        
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
                const { removeToken } = useAuthToken();
                removeToken();
                await navigateTo('/_/auth/login');
            }
            throw error;
        }
    };
    return { authFetch };
};