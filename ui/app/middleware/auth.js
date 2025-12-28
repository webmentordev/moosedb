export default defineNuxtRouteMiddleware((to, from) => {
    const { getToken } = useAuthToken();
    const token = getToken();
    
    if (!token) {
        return navigateTo('/_/auth/login');
    }else{
        return navigateTo('/_/');
    }
});