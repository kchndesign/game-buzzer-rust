import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import AdminRoute from './routes/AdminRoute';
import PlayerRoute from './routes/PlayerRoute';

const router = createBrowserRouter([
    // game code input page
    {
        path: '/:code?',
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
