import React, { useEffect } from 'react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { invoke } from '@tauri-apps/api/core';
import { closeWindow } from '@/lib/windowUtils';

export default function Consent() {
  console.log('Consent component rendered');

  // Handle window close events
  useEffect(() => {
    console.log('Consent component mounted');
    const handleBeforeUnload = () => {
      // If user closes window without making a choice, decline by default
      invoke('decline_analytics_consent').catch(console.error);
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  const handleAccept = async () => {
    console.log('Accepting analytics consent');
    try {
      await invoke('accept_analytics_consent');
      // Add a small delay before closing to ensure the event is sent
      setTimeout(() => {
        closeWindow();
      }, 100);
    } catch (error) {
      console.error('Failed to accept analytics consent:', error);
      closeWindow();
    }
  };

  const handleDecline = async () => {
    console.log('Declining analytics consent');
    try {
      await invoke('decline_analytics_consent');
      // Add a small delay before closing to ensure the event is sent
      setTimeout(() => {
        closeWindow();
      }, 100);
    } catch (error) {
      console.error('Failed to decline analytics consent:', error);
      closeWindow();
    }
  };

  return (
    <div className="h-full bg-background flex items-center justify-center p-4 " data-tauri-drag-region>
      <Card className="w-full max-w-md" data-tauri-drag-region>
        <CardHeader className="text-center" data-tauri-drag-region>
          <CardTitle className="text-xl">Help us improve Holdem</CardTitle>
          <CardDescription data-tauri-drag-region>
            We'd like to collect anonymous usage data to improve your experience
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6" data-tauri-drag-region>
          <div className="text-sm text-muted-foreground space-y-3">
            <div>
              <p className="font-medium mb-2">We collect:</p>
              <ul className="list-disc list-inside space-y-1 ml-4">
                <li>App launch events</li>
                <li>File drag and drop usage</li>
                <li>Basic app performance metrics</li>
              </ul>
            </div>
            <div>
              <p className="font-medium mb-2">We do NOT collect:</p>
              <ul className="list-disc list-inside space-y-1 ml-4">
                <li>Personal information</li>
                <li>File contents or names</li>
                <li>Your browsing history</li>
              </ul>
            </div>
          </div>
          <div className="flex gap-3 pt-4">
            <Button 
              variant="outline" 
              onClick={handleDecline}
              className="flex-1"
            >
              Decline
            </Button>
            <Button 
              onClick={handleAccept}
              className="flex-1"
            >
              Accept
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
} 