import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import AdminRoute from './routes/AdminRoute';
import PlayerRoute from './routes/PlayerRoute';
import PlayerCodeInput from './routes/PlayerCodeInput';

const router = createBrowserRouter([
    // player input code page
    {
        path: '/',
        element: <PlayerCodeInput />,
    },

    // player game page
    {
        path: '/:code',
        element: <PlayerRoute />,
    },

    // admin default page
    {
        path: '/create',
        element: <AdminRoute />,
    },
]);

function App() {
    return (
        <div className="App">
            <h1>Hello world</h1>
            <RouterProvider router={router} />
        </div>
    );
}

export default App;
