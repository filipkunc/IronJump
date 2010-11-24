//
//  FPOpenGLView.m
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameView.h"

@implementation FPGameView

@synthesize game;

- (id)initWithCoder:(NSCoder *)coder
{
	self = [super initWithCoder:coder];
    if (self) 
	{
		[self setupSharedContext];
		glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
		glEnable(GL_TEXTURE_2D);
		glEnable(GL_BLEND);
		glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);	
		
		timer = [NSTimer scheduledTimerWithTimeInterval:1.0 / 60.0
												 target:self 
											   selector:@selector(gameLoop)
											   userInfo:nil
												repeats:YES];
		
		pressedKeys = [[NSMutableSet alloc] init];
        replay = [[FPReplay alloc] init];
        replayingGame = NO;
	}
    return self;
}

- (void)dealloc
{
	[game release];
    [replay release];
	[pressedKeys release];
	[super dealloc];
}

- (BOOL)isFlipped
{
	return YES;
}

- (void)reshape
{
	[[self openGLContext] makeCurrentContext];
	[self setNeedsDisplay:YES];
}

- (void)playWithKeyboard
{
    replayingGame = NO;
    [replay release];
    replay = [FPReplay new];
}

- (void)playFromRecord
{
    replayingGame = YES;
    [replay resetReplay];
}

- (void)gameLoop
{
    if (replayingGame)
    {
        [replay playFrameToGame:game];
    }
    else
    {
        CGPoint inputAcceleration = CGPointZero;
        if ([pressedKeys containsObject:[NSNumber numberWithUnsignedShort:NSLeftArrowFunctionKey]])
            inputAcceleration.x = -1.0f;
        else if ([pressedKeys containsObject:[NSNumber numberWithUnsignedShort:NSRightArrowFunctionKey]])
            inputAcceleration.x = 1.0f;
        if ([pressedKeys containsObject:[NSNumber numberWithUnsignedShort:NSUpArrowFunctionKey]])
            inputAcceleration.y = 1.0f;
        
        [game setInputAcceleration:inputAcceleration];
        [game update];
        //[replay addFrameFromGame:game];
    }
	[self setNeedsDisplay:YES];
}

- (void)drawRect:(NSRect)dirtyRect 
{
	[self reshapeFlippedOrtho2D];
	
	NSRect rect = [self bounds];
	
	glClear(GL_COLOR_BUFFER_BIT);
	glLoadIdentity();
	glTranslatef(0, rect.size.height, 0);
	glScalef(1, -1, 1);
	
	[game draw];
	
	[[self openGLContext] flushBuffer];
}

#pragma mark Key Processing

- (BOOL)acceptsFirstResponder
{
	return YES;
}

- (void)removeAllPressedKeys
{
	[pressedKeys removeAllObjects];
}

- (void)processChar:(unichar)character isKeyDown:(BOOL)keyDown
{
	if (keyDown)
		[pressedKeys addObject:[NSNumber numberWithUnsignedShort:character]];		
	else
		[pressedKeys removeObject:[NSNumber numberWithUnsignedShort:character]];		
}

- (void)processKeyEvent:(NSEvent *)theEvent isKeyDown:(BOOL)keyDown
{
	NSString *characters = [theEvent characters];
	if ([characters length] == 1 && ![theEvent isARepeat])
	{
		unichar character = [characters characterAtIndex:0];
		[self processChar:character isKeyDown:keyDown];
	}
}

- (void)keyDown:(NSEvent *)theEvent
{
	[self processKeyEvent:theEvent isKeyDown:YES];
}

- (void)keyUp:(NSEvent *)theEvent
{
	[self processKeyEvent:theEvent isKeyDown:NO];
}

@end
