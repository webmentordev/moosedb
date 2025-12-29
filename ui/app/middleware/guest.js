export default defineNuxtRouteMiddleware((to, from) => {
    const { getToken } = useAuthToken();
    const token = getToken();
    
    if (token) {
        return navigateTo('/_/collections');
    }
});